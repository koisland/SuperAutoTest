use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum PetName {
    Ant,
    Badger,
    Beaver,
    Bison,
    Blowfish,
    Boar,
    Camel,
    Cat,
    Cow,
    Crab,
    Cricket,
    Crocodile,
    Deer,
    Dodo,
    Dog,
    Dolphin,
    Dragon,
    Duck,
    Elephant,
    Fish,
    Flamingo,
    Fly,
    Giraffe,
    Gorilla,
    Hedgehog,
    Hippo,
    Horse,
    Kangaroo,
    Leopard,
    Mammoth,
    Monkey,
    Mosquito,
    Otter,
    Ox,
    Parrot,
    Peacock,
    Penguin,
    Pig,
    Rabbit,
    Rat,
    Rhino,
    Rooster,
    Scorpion,
    Seal,
    Shark,
    Sheep,
    Shrimp,
    Skunk,
    Sloth,
    Snail,
    Snake,
    Spider,
    Squirrel,
    Swan,
    Tiger,
    Turkey,
    Turtle,
    Whale,
    Worm,
    ZombieCricket,
    Ram,
    Bee,
    DirtyRat,
}

impl std::fmt::Display for PetName {
    #[cfg(not(tarpaulin_include))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PetName::Ant => write!(f, "Ant"),
            PetName::Badger => write!(f, "Badger"),
            PetName::Beaver => write!(f, "Beaver"),
            PetName::Bison => write!(f, "Bison"),
            PetName::Blowfish => write!(f, "Blowfish"),
            PetName::Boar => write!(f, "Boar"),
            PetName::Camel => write!(f, "Camel"),
            PetName::Cat => write!(f, "Cat"),
            PetName::Cow => write!(f, "Cow"),
            PetName::Crab => write!(f, "Crab"),
            PetName::Cricket => write!(f, "Cricket"),
            PetName::Crocodile => write!(f, "Crocodile"),
            PetName::Deer => write!(f, "Deer"),
            PetName::Dodo => write!(f, "Dodo"),
            PetName::Dog => write!(f, "Dog"),
            PetName::Dolphin => write!(f, "Dolphin"),
            PetName::Dragon => write!(f, "Dragon"),
            PetName::Duck => write!(f, "Duck"),
            PetName::Elephant => write!(f, "Elephant"),
            PetName::Fish => write!(f, "Fish"),
            PetName::Flamingo => write!(f, "Flamingo"),
            PetName::Fly => write!(f, "Fly"),
            PetName::Giraffe => write!(f, "Giraffe"),
            PetName::Gorilla => write!(f, "Gorilla"),
            PetName::Hedgehog => write!(f, "Hedgehog"),
            PetName::Hippo => write!(f, "Hippo"),
            PetName::Horse => write!(f, "Horse"),
            PetName::Kangaroo => write!(f, "Kangaroo"),
            PetName::Leopard => write!(f, "Leopard"),
            PetName::Mammoth => write!(f, "Mammoth"),
            PetName::Monkey => write!(f, "Monkey"),
            PetName::Mosquito => write!(f, "Mosquito"),
            PetName::Otter => write!(f, "Otter"),
            PetName::Ox => write!(f, "Ox"),
            PetName::Parrot => write!(f, "Parrot"),
            PetName::Peacock => write!(f, "Peacock"),
            PetName::Penguin => write!(f, "Penguin"),
            PetName::Pig => write!(f, "Pig"),
            PetName::Rabbit => write!(f, "Rabbit"),
            PetName::Rat => write!(f, "Rat"),
            PetName::Rhino => write!(f, "Rhino"),
            PetName::Rooster => write!(f, "Rooster"),
            PetName::Scorpion => write!(f, "Scorpion"),
            PetName::Seal => write!(f, "Seal"),
            PetName::Shark => write!(f, "Shark"),
            PetName::Sheep => write!(f, "Sheep"),
            PetName::Shrimp => write!(f, "Shrimp"),
            PetName::Skunk => write!(f, "Skunk"),
            PetName::Sloth => write!(f, "Sloth"),
            PetName::Snail => write!(f, "Snail"),
            PetName::Snake => write!(f, "Snake"),
            PetName::Spider => write!(f, "Spider"),
            PetName::Squirrel => write!(f, "Squirrel"),
            PetName::Swan => write!(f, "Swan"),
            PetName::Tiger => write!(f, "Tiger"),
            PetName::Turkey => write!(f, "Turkey"),
            PetName::Turtle => write!(f, "Turtle"),
            PetName::Whale => write!(f, "Whale"),
            PetName::Worm => write!(f, "Worm"),
            PetName::ZombieCricket => write!(f, "Zombie Cricket"),
            PetName::Ram => write!(f, "Ram"),
            PetName::Bee => write!(f, "Bee"),
            PetName::DirtyRat => write!(f, "Dirty Rat"),
        }
    }
}
