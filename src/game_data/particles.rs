const PARTCLE_DATA: &[u8] = include_bytes!("./assets/particle_tables.bin");

pub const NUM_PARTICLE_SETS: usize = 51;

#[derive(Clone)]
pub struct Particle {
    pub x: i8,
    pub y: i8,
    pub color: u8,
}

#[derive(Clone, Default)]
pub struct ParticleSet {
    pub count: usize,
    pub offset: usize,
}

#[derive(Clone)]
pub struct ParticleSets {
    pub particle_sets: Vec<ParticleSet>,
    pub particles: Vec<Particle>,
}

pub fn load_particle_sets() -> ParticleSets {
    let color_table: &[u8] = PARTCLE_DATA;
    let particle_tables: &[u8] = &PARTCLE_DATA[16..];

    let mut iparticle: usize = 0;
    let mut cursor: usize = 0;

    let mut particle_sets: Vec<ParticleSet> = vec![Default::default(); NUM_PARTICLE_SETS];
    for particle_set in particle_sets.iter_mut() {
        let offset = iparticle;

        for _ in 0..80 {
            let x = particle_tables[cursor] as i8;
            cursor += 2;

            if x != -1 {
                iparticle += 1;
            }
        }

        *particle_set = ParticleSet {
            offset,
            count: iparticle - offset,
        };
    }

    let mut particles: Vec<Particle> = Vec::with_capacity(iparticle);

    cursor = 0;
    for iparticle in 0..80 * NUM_PARTICLE_SETS {
        let x = PARTCLE_DATA[cursor] as i8;
        cursor += 1;

        let y = PARTCLE_DATA[cursor] as i8;
        cursor += 1;

        if x == -1 {
            continue;
        }

        let color = color_table[(iparticle % 80) & 0x0f];
        particles.push(Particle { x, y, color });
    }

    ParticleSets {
        particle_sets,
        particles,
    }
}
