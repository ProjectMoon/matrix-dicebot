use rand::prelude::*;
use crate::dice;

pub trait Roll {
    type Output;

    fn roll(&self) -> Self::Output;
}

impl Roll for dice::Dice {
    type Output = u32; 

    fn roll(&self) -> u32 {
        let mut rng = rand::thread_rng();
        (0..self.count).map(|_| rng.gen_range(1, self.sides + 1)).sum()
    }
}

impl Roll for dice::Element {
    type Output = u32; 

    fn roll(&self) -> u32 {
        match self {
            dice::Element::Dice(d) => d.roll(),
            dice::Element::Bonus(b) => *b,
        }
    }
}

impl Roll for dice::SignedElement {
    type Output = i32; 

    fn roll(&self) -> i32 {
        match self {
            dice::SignedElement::Positive(e) => e.roll() as i32,
            dice::SignedElement::Negative(e) => -(e.roll() as i32),
        }
    }
}
