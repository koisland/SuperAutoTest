use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::error::SAPTestError;

/// Possible names for [`Pet`](crate::pets::pet::Pet)s.
#[allow(missing_docs)]
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
    Bus,
    Chick,
    ZombieFly,
    Beetle,
    Bluebird,
    Chinchilla,
    Cockroach,
    Duckling,
    FrilledDragon,
    Frog,
    Hummingbird,
    Iguana,
    Kiwi,
    Ladybug,
    Marmoset,
    Moth,
    Mouse,
    Pillbug,
    Seahorse,
    Butterfly,
    Bat,
    AtlanticPuffin,
    Dove,
    Koala,
    Panda,
    Pug,
    Stork,
    Racoon,
    Toucan,
    Wombat,
    Aardvark,
    Bear,
    Seagull,
    Blobfish,
    Clownfish,
    Toad,
    Woodpecker,
    /// A custom [`PetName`].
    Custom(String),
}

impl FromStr for PetName {
    type Err = SAPTestError;

    #[cfg(not(tarpaulin_include))]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Ant" => Ok(PetName::Ant),
            "Badger" => Ok(PetName::Badger),
            "Beaver" => Ok(PetName::Beaver),
            "Bison" => Ok(PetName::Bison),
            "Blowfish" => Ok(PetName::Blowfish),
            "Boar" => Ok(PetName::Boar),
            "Camel" => Ok(PetName::Camel),
            "Cat" => Ok(PetName::Cat),
            "Cow" => Ok(PetName::Cow),
            "Crab" => Ok(PetName::Crab),
            "Cricket" => Ok(PetName::Cricket),
            "Crocodile" => Ok(PetName::Crocodile),
            "Deer" => Ok(PetName::Deer),
            "Dodo" => Ok(PetName::Dodo),
            "Dog" => Ok(PetName::Dog),
            "Dolphin" => Ok(PetName::Dolphin),
            "Dragon" => Ok(PetName::Dragon),
            "Duck" => Ok(PetName::Duck),
            "Elephant" => Ok(PetName::Elephant),
            "Fish" => Ok(PetName::Fish),
            "Flamingo" => Ok(PetName::Flamingo),
            "Fly" => Ok(PetName::Fly),
            "Giraffe" => Ok(PetName::Giraffe),
            "Gorilla" => Ok(PetName::Gorilla),
            "Hedgehog" => Ok(PetName::Hedgehog),
            "Hippo" => Ok(PetName::Hippo),
            "Horse" => Ok(PetName::Horse),
            "Kangaroo" => Ok(PetName::Kangaroo),
            "Leopard" => Ok(PetName::Leopard),
            "Mammoth" => Ok(PetName::Mammoth),
            "Monkey" => Ok(PetName::Monkey),
            "Mosquito" => Ok(PetName::Mosquito),
            "Otter" => Ok(PetName::Otter),
            "Ox" => Ok(PetName::Ox),
            "Parrot" => Ok(PetName::Parrot),
            "Peacock" => Ok(PetName::Peacock),
            "Penguin" => Ok(PetName::Penguin),
            "Pig" => Ok(PetName::Pig),
            "Rabbit" => Ok(PetName::Rabbit),
            "Rat" => Ok(PetName::Rat),
            "Rhino" => Ok(PetName::Rhino),
            "Rooster" => Ok(PetName::Rooster),
            "Scorpion" => Ok(PetName::Scorpion),
            "Seal" => Ok(PetName::Seal),
            "Shark" => Ok(PetName::Shark),
            "Sheep" => Ok(PetName::Sheep),
            "Shrimp" => Ok(PetName::Shrimp),
            "Skunk" => Ok(PetName::Skunk),
            "Sloth" => Ok(PetName::Sloth),
            "Snail" => Ok(PetName::Snail),
            "Snake" => Ok(PetName::Snake),
            "Spider" => Ok(PetName::Spider),
            "Squirrel" => Ok(PetName::Squirrel),
            "Swan" => Ok(PetName::Swan),
            "Tiger" => Ok(PetName::Tiger),
            "Turkey" => Ok(PetName::Turkey),
            "Turtle" => Ok(PetName::Turtle),
            "Whale" => Ok(PetName::Whale),
            "Worm" => Ok(PetName::Worm),
            "Beetle" => Ok(PetName::Beetle),
            "Bluebird" => Ok(PetName::Bluebird),
            "Chinchilla" => Ok(PetName::Chinchilla),
            "Cockroach" => Ok(PetName::Cockroach),
            "Duckling" => Ok(PetName::Duckling),
            "Frilled Dragon" => Ok(PetName::FrilledDragon),
            "Frog" => Ok(PetName::Frog),
            "Hummingbird" => Ok(PetName::Hummingbird),
            "Iguana" => Ok(PetName::Iguana),
            "Kiwi" => Ok(PetName::Kiwi),
            "Ladybug" => Ok(PetName::Ladybug),
            "Marmoset" => Ok(PetName::Marmoset),
            "Moth" => Ok(PetName::Moth),
            "Mouse" => Ok(PetName::Mouse),
            "Pillbug" => Ok(PetName::Pillbug),
            "Seahorse" => Ok(PetName::Seahorse),
            "Butterfly" => Ok(PetName::Butterfly),
            "Zombie Cricket" => Ok(PetName::ZombieCricket),
            "Ram" => Ok(PetName::Ram),
            "Bee" => Ok(PetName::Bee),
            "Dirty Rat" => Ok(PetName::DirtyRat),
            "Bus" => Ok(PetName::Bus),
            "Chick" => Ok(PetName::Chick),
            "Zombie Fly" => Ok(PetName::ZombieFly),
            "Bat" => Ok(PetName::Bat),
            "Atlantic Puffin" => Ok(PetName::AtlanticPuffin),
            "Dove" => Ok(PetName::Dove),
            "Koala" => Ok(PetName::Koala),
            "Panda" => Ok(PetName::Panda),
            "Pug" => Ok(PetName::Pug),
            "Stork" => Ok(PetName::Stork),
            "Racoon" => Ok(PetName::Racoon),
            "Toucan" => Ok(PetName::Toucan),
            "Wombat" => Ok(PetName::Wombat),
            "Aardvark" => Ok(PetName::Aardvark),
            "Bear" => Ok(PetName::Bear),
            "Seagull" => Ok(PetName::Seagull),
            "Blobfish" => Ok(PetName::Blobfish),
            "Clownfish" => Ok(PetName::Clownfish),
            "Toad" => Ok(PetName::Toad),
            "Woodpecker" => Ok(PetName::Woodpecker),
            _ => Ok(PetName::Custom(s.to_string())),
        }
    }
}

impl Display for PetName {
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
            PetName::Bus => write!(f, "Bus"),
            PetName::Chick => write!(f, "Chick"),
            PetName::ZombieFly => write!(f, "Zombie Fly"),
            PetName::Beetle => write!(f, "Beetle"),
            PetName::Bluebird => write!(f, "Bluebird"),
            PetName::Chinchilla => write!(f, "Chinchilla"),
            PetName::Cockroach => write!(f, "Cockroach"),
            PetName::Duckling => write!(f, "Duckling"),
            PetName::FrilledDragon => write!(f, "Frilled Dragon"),
            PetName::Frog => write!(f, "Frog"),
            PetName::Hummingbird => write!(f, "Hummingbird"),
            PetName::Iguana => write!(f, "Iguana"),
            PetName::Kiwi => write!(f, "Kiwi"),
            PetName::Ladybug => write!(f, "Ladybug"),
            PetName::Marmoset => write!(f, "Marmoset"),
            PetName::Moth => write!(f, "Moth"),
            PetName::Mouse => write!(f, "Mouse"),
            PetName::Pillbug => write!(f, "Pillbug"),
            PetName::Seahorse => write!(f, "Seahorse"),
            PetName::Butterfly => write!(f, "Butterfly"),
            PetName::Bat => write!(f, "Bat"),
            PetName::AtlanticPuffin => write!(f, "Atlantic Puffin"),
            PetName::Dove => write!(f, "Dove"),
            PetName::Koala => write!(f, "Koala"),
            PetName::Panda => write!(f, "Panda"),
            PetName::Pug => write!(f, "Pug"),
            PetName::Stork => write!(f, "Stork"),
            PetName::Racoon => write!(f, "Racoon"),
            PetName::Toucan => write!(f, "Toucan"),
            PetName::Wombat => write!(f, "Wombat"),
            PetName::Aardvark => write!(f, "Aardvark"),
            PetName::Bear => write!(f, "Bear"),
            PetName::Seagull => write!(f, "Seagull"),
            PetName::Blobfish => write!(f, "Blobfish"),
            PetName::Clownfish => write!(f, "Clownfish"),
            PetName::Toad => write!(f, "Toad"),
            PetName::Woodpecker => write!(f, "Woodpecker"),
            PetName::Custom(name) => write!(f, "{name}"),
        }
    }
}
