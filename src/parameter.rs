use rand::seq::SliceRandom;

pub trait Parameter {
    fn get_next(&mut self) -> f32;
}


// Static
pub struct StaticParameter {
    value: f32
}

impl StaticParameter {
    pub fn from_val(val: f32) -> Self {
        StaticParameter {
            value: val,
        }
    }
}

impl Parameter for StaticParameter {    
    fn get_next(&mut self) -> f32 {
        self.value
    }
}

////////////
// RANDOM //
////////////

pub struct ChooseParameter {
    items: Vec<f32>
}

impl ChooseParameter {
    pub fn from_seq(seq: &Vec<f32>) -> Self {
        ChooseParameter {
            items: seq.to_vec(),
        }
    }
}

impl Parameter for ChooseParameter {    
    fn get_next(&mut self) -> f32 {
        match self.items.choose(&mut rand::thread_rng()) {
            Some(thing) => *thing,
            None => 0.0         
        }
    }
}

////////////
// CYCLE  //
////////////

pub struct CycleParameter {
    items: Vec<f32>,
    index: usize,        
}

impl CycleParameter {
    pub fn from_seq(seq: &Vec<f32>) -> Self {
        CycleParameter {
            items: seq.to_vec(),
            index: 0,
        }
    }
}

impl Parameter for CycleParameter {    
    fn get_next(&mut self) -> f32 {
        let item = self.items[self.index];

        self.index += 1;
        
        if self.index >= self.items.len() {
            self.index = 0;
        }
                
        item       
    }
}

//////////
// RAMP //
//////////

pub struct RampParameter {
    min: f32,
    inc: f32,
    steps: f32,
    step_count: f32,
}

impl RampParameter {
    pub fn from_params(min: f32, max: f32, steps: f32) -> Self {        
        RampParameter {            
            min: min,
            inc: (max - min) / steps,
            steps: steps,
            step_count: (0.0).into(),
        }
    }
}

impl Parameter for RampParameter {    
    fn get_next(&mut self) -> f32 {
        let cur = self.min + self.step_count * self.inc;
        self.step_count = self.step_count + 1.0;
        if self.step_count > self.steps {
            self.step_count = (0.0).into();
        }
        cur
    }
}

////////////
// BOUNCE //
////////////

// sinusoidal bounce
pub struct BounceParameter {
    min: f32,
    degree_inc: f32,
    range: f32,
    steps: f32,
    step_count: f32,
}

impl BounceParameter {
    pub fn from_params(min: f32, max: f32, steps: f32) -> Self {
        let mut dec_inc:f32 = (360.0).into();
        dec_inc = dec_inc / steps;
        BounceParameter {                        
            min: min,
            range: max - min,
            degree_inc: dec_inc,            
            steps: steps,
            step_count: (0.0).into(),
        }
    }
}

impl Parameter for BounceParameter {    
    fn get_next(&mut self) -> f32 {
        // why doesn't rust has a hashable float ?????
        let deg_inc_raw:f32 = self.degree_inc.into();
        let mut step_count_raw:f32 = self.step_count.into();
        let steps_raw:f32 = self.steps.into();
        let min_raw:f32 = self.min.into();
        let range_raw:f32 = self.range.into();
                
        let degree:f32 = (deg_inc_raw * (step_count_raw % steps_raw)) % 360.0;
        let abs_sin:f32 = degree.to_radians().sin().abs().into();
        
        let cur:f32 = min_raw + (abs_sin * range_raw);

        step_count_raw += 1.0;
        self.step_count = step_count_raw.into(); 
        
        cur.into()
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
        
    #[test]
    fn test_bounce_gen() {
        let mut bounce_gen = BounceParameter::from_params((20.0).into(), (200.0).into(), (10.0).into());
        let mut results = Vec::new();
        for _ in 0..10 {
            results.push(bounce_gen.get_next());
        }
        println!("Result: {:?}", results);
    }

    #[test]
    fn test_ramp_gen() {
        let mut ramp_gen = RampParameter::from_params((20.0).into(), (200.0).into(), (10.0).into());
        let mut results = Vec::new();
        for _ in 0..10 {
            results.push(ramp_gen.get_next());
        }
        println!("Result: {:?}", results);
    }
}
