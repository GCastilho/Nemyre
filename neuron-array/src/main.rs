use rand::{random_iter, random_range};
use std::{
    collections::VecDeque,
    thread::sleep,
    time::{Duration, Instant},
};

struct Coordinates {
    x: f64,
    y: f64,
    z: f64,
}

/// Conexão entre neurônios (e de neurônios <-> nervos)
struct Synapse {
    with: usize,
    weight: f64,
}

struct PreviousAction {
    with: usize,
    tick: u64,
}

struct Neuron {
    potential: f64,
    previous_actions: VecDeque<PreviousAction>,
    coordinates: Coordinates,
    axon: Vec<Synapse>,
}

enum NerveKind {
    Input,
    Output,
}

/// Nervos controlam o I/O
///
/// A ideia é que eles convertem excitação para pulsos no Input e pulsos para excitação no Output.
/// Por hora isso será uma função linear sem uma easing function, que deve ser implementada depois
struct Nerve {
    kind: NerveKind,
    excitation: u8,        // TODO: Esse é o melhor nome?
    ticks_since_fire: u64, // TODO: Definitivamente precisa de nome melhor
    coordinates: Coordinates,
    axon: Vec<Synapse>,
}

fn main() {
    let mut neurons = Vec::new();
    for _ in 0..1000 {
        let coordinates = Coordinates {
            x: random_range(0.0..10.0),
            y: random_range(0.0..10.0),
            z: random_range(0.0..10.0),
        };
        let mut axon = random_iter().take(110).collect::<Vec<u64>>();
        axon.dedup();
        let axon = axon
            .into_iter()
            .map(|with| Synapse {
                with: with
                    .try_into()
                    .expect("must be able to convert u64 to usize"),
                weight: random_range(0.0..1.0),
            })
            .collect();

        let neuron = Neuron {
            potential: 0.0,
            previous_actions: VecDeque::new(),
            coordinates,
            axon,
        };
        neurons.push(neuron);
    }

    let mut tick: u64 = 0;
    let ticks_per_second = 1000;
    let tick_min_duration = Duration::from_millis(ticks_per_second / 1000);
    loop {
        tick += 1;
        let now = Instant::now();

        let tick_duration_left = tick_min_duration
            .checked_sub(now.elapsed())
            .unwrap_or_default();
        sleep(tick_duration_left);
    }
}
