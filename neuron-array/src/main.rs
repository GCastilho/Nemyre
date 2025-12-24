#![allow(dead_code)] // TODO: Eventually this should be removed; I've added bc it was annoying

use rand::{random_range, seq::index::sample};
use std::{
    collections::{HashMap, VecDeque},
    ops::{Index, IndexMut},
    thread::sleep,
    time::{Duration, Instant},
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct ReceptorId(usize);

impl<T> Index<ReceptorId> for Vec<T> {
    type Output = T;

    fn index(&self, index: ReceptorId) -> &Self::Output {
        &self[index.0]
    }
}

impl<T> IndexMut<ReceptorId> for Vec<T> {
    fn index_mut(&mut self, index: ReceptorId) -> &mut Self::Output {
        &mut self[index.0]
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct NeuronId(usize);

impl<T> Index<NeuronId> for Vec<T> {
    type Output = T;

    fn index(&self, index: NeuronId) -> &Self::Output {
        &self[index.0]
    }
}

impl<T> IndexMut<NeuronId> for Vec<T> {
    fn index_mut(&mut self, index: NeuronId) -> &mut Self::Output {
        &mut self[index.0]
    }
}

const TICKS_PER_SECOND: u32 = 1000;
const MINIMUM_POTENTIAL: f64 = 100.0;

struct Coordinates {
    x: f64,
    y: f64,
    z: f64,
}

// TODO: Update doc
/// Signal received on last tick
#[derive(Debug, Clone)]
struct Action {
    with: SenderId,
    value: f64,
}

/// Action processed/received in a previous tick, kept for Synapse adjustment upon new Action
struct PreviousAction {
    with: SenderId,
    tick: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SenderId {
    Receptor(ReceptorId),
    Neuron(NeuronId),
}

/// Mantém as conexões entre os neurônios (e de neurônios <-> nervos) assim como a força da conexão para o recebedor (ω)
struct Dispatcher {
    // SenderId (Receptor or Neuron), Receiver, receiver strength (ω)
    connections: HashMap<(SenderId, NeuronId), f64>,
    // Vec of inboxes, one for each neuron
    inboxes: Vec<Vec<Action>>,
}

impl Dispatcher {
    pub fn new(neuron_count: usize) -> Self {
        Self {
            connections: HashMap::new(),
            inboxes: vec![Vec::new(); neuron_count],
        }
    }

    pub fn connect(&mut self, from: SenderId, to: NeuronId, weight: f64) {
        self.connections.insert((from, to), weight);
    }

    pub fn disconnect(&mut self, from: SenderId, to: NeuronId) {
        self.connections.remove(&(from, to));
    }

    // TODO: Seria ideal não ter q iterar por todos, um tempo log é melhor q linear. HasMap<HashMap<f64>>?
    // Vec<Vec<f64>>, q pode ser feito usando só um Vec, permite tempo constante mas gasta mto espaço pq
    // poucos neurônios se conectam com outros neurônios pq é uma conexão local na maioria das vezes
    pub fn send(&mut self, from: SenderId, value: f64) {
        for (&(f, t), &weight) in self.connections.iter() {
            if f == from {
                self.inboxes[t].push(Action {
                    with: from,
                    value: value * weight,
                });
            }
        }
    }

    // TODO: Retornar um iterator (usando internal mutability) em um VecDeque evita realocação
    pub fn drain_inbox(&mut self, id: NeuronId) -> Vec<Action> {
        std::mem::take(&mut self.inboxes[id])
    }
}

/// Input receptor from the system
///
/// Converts excitation value into synapses to connected neurons. For now that is linear
/// TODO: Será necessário poder setar a frequência ou os pulsos diretamente (ex: pulsos randômicos)
struct SensoryReceptor {
    id: ReceptorId,
    excitation: u8, // TODO: Esse é o melhor nome?
    coordinates: Coordinates,
    last_tick_fired: u64,
    min_period: u32,
}

impl SensoryReceptor {
    // TODO: max_frequency customizável por neurônio assim como a forma de onda
    pub fn new(id: usize, max_frequency: u32, coordinates: Coordinates) -> Self {
        Self {
            id: ReceptorId(id),
            excitation: 0,
            min_period: TICKS_PER_SECOND / max_frequency, // TODO: Talvez arredondar, não truncar
            coordinates,
            last_tick_fired: 0,
        }
    }

    // TODO: O ideal é que o tick seja acessível globalmente de alguma forma
    // TODO: N sei se update é o melhor nome
    // TODO: Add easing function to change period smoothly
    /// Process receptor excitation into an action strength
    ///
    /// Returns [`None`] if receptor shouldn't fire
    /// Returns [`Some(t)`] with the strength the receptor fired the Action Potential
    fn update(&mut self, tick: u64) -> Option<f64> {
        match self.excitation {
            0 => None,
            n => {
                let period = (u8::MAX as u32 * self.min_period) as f64 / n as f64;
                Some(period.round() as u64)
            }
        }
        .filter(|period| self.last_tick_fired + period <= tick)
        .map(|_| {
            self.last_tick_fired = tick;
            // TODO: Implementar disparo. Da onde vem a força?
            1.0
        })
    }
}

struct Neuron {
    id: NeuronId,
    potential: f64,
    previous_actions: VecDeque<PreviousAction>,
    coordinates: Coordinates,
}

impl Neuron {
    // TODO: A força da conexão é controlada pelo recebedor, não por quem envia. Quem envia controla
    // a força do disparo
    pub fn update(&mut self, tick: u64, actions: &[Action]) -> Option<f64> {
        let potential_received = actions.iter().map(|a| a.value).sum::<f64>();
        self.potential += potential_received;

        if self.potential < MINIMUM_POTENTIAL {
            return None;
        }

        actions
            .iter()
            .map(|a| PreviousAction { with: a.with, tick })
            .for_each(|pa| self.previous_actions.push_back(pa));

        // Calculado "de qualquer jeito" para não usar 1.0
        let strength = if self.potential <= 0.0 {
            0.0
        } else {
            // Satura em 1 conforme `potential` cresce (1 - e^{-x})
            let s = 1.0 - (-self.potential).exp();
            s.clamp(0.0, 1.0)
        };
        self.potential = -0.1; // Equivalente a despolarizar. Acho q faz mais sentido isso tbm recuperar com os ticks
        Some(strength)
    }
}

/// Output neurons from the system
///
/// Receive synapses and convert that into an excitation value
struct MotorNerve {
    excitation: u8, // TODO: Esse é o melhor nome?
    coordinates: Coordinates,
}

fn main() {
    let mut receptors = Vec::new();
    for id in 0..110 {
        let coordinates = Coordinates {
            x: random_range(0.0..10.0),
            y: random_range(0.0..10.0),
            z: random_range(0.0..10.0),
        };
        let receptor = SensoryReceptor::new(id, 20, coordinates);
        receptors.push(receptor);
    }

    let neuron_count = 1000;
    let mut neurons = Vec::new();
    for id in 0..neuron_count {
        let coordinates = Coordinates {
            x: random_range(0.0..10.0),
            y: random_range(0.0..10.0),
            z: random_range(0.0..10.0),
        };

        let neuron = Neuron {
            id: NeuronId(id),
            potential: 0.0,
            previous_actions: VecDeque::new(),
            coordinates,
        };
        neurons.push(neuron);
    }

    let senders = receptors
        .iter()
        .map(|r| SenderId::Receptor(r.id))
        .chain(neurons.iter().map(|n| SenderId::Neuron(n.id)));
    let mut dispatcher = Dispatcher::new(neuron_count);
    for sender in senders {
        let sampled = sample(
            &mut rand::rng(),
            neuron_count,
            (0.15 * neuron_count as f64) as usize, // Connected to 15% of each
        );
        for neuron_to in sampled.into_iter() {
            dispatcher.connect(sender, NeuronId(neuron_to), random_range(0.0..1.0));
        }
    }

    let mut tick: u64 = 0;
    let tick_min_duration = Duration::from_millis(TICKS_PER_SECOND as u64 / 1000);
    loop {
        tick += 1;
        let now = Instant::now();

        // Seta a excitação dos receptores
        // Atualiza os receptores
        for receptor in &mut receptors {
            receptor.excitation = random_range(118..138);
            if let Some(strength) = receptor.update(tick) {
                // O valor do Potencial de Ação é enviado para o with. Isso é a "força do disparo"
                dispatcher.send(SenderId::Receptor(receptor.id), strength);
            }
        }

        // Processa os neurônios
        for neuron in &mut neurons {
            let actions = dispatcher.drain_inbox(neuron.id);
            if let Some(strength) = neuron.update(tick, &actions) {
                dispatcher.send(SenderId::Neuron(neuron.id), strength);
            }
        }

        let tick_duration_left = tick_min_duration
            .checked_sub(now.elapsed())
            .unwrap_or_default();
        sleep(tick_duration_left);
    }
}

#[cfg(test)]
mod sensory_tests {
    use crate::{Coordinates, SensoryReceptor};

    const COORDINATES: Coordinates = Coordinates {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    #[test]
    fn fire_on_max_excitation() {
        let max_frequency = 20;
        let mut receptor = SensoryReceptor::new(1, max_frequency, COORDINATES);
        receptor.excitation = 255;

        assert!(receptor.update(1).is_none());
        assert!(receptor.update(50).is_some());
        assert_eq!(receptor.last_tick_fired, 50);
    }

    #[test]
    fn fire_half_excitation() {
        let max_frequency = 20;
        let mut receptor = SensoryReceptor::new(1, max_frequency, COORDINATES);
        receptor.excitation = 128;

        assert!(receptor.update(50).is_none());
        assert!(receptor.update(100).is_some());
        assert_eq!(receptor.last_tick_fired, 100);
    }

    #[test]
    fn fire_74_excitation() {
        let max_frequency = 20;
        let mut receptor = SensoryReceptor::new(1, max_frequency, COORDINATES);
        receptor.excitation = 74;

        assert!(receptor.update(171).is_none());
        assert!(receptor.update(172).is_some());
    }
}
