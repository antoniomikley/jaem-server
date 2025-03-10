use rand::{seq::SliceRandom, Rng};

/// Struct holding the necessary information to generate a somewhat unique link consisting of an
/// adjective, an animal and a four digit random number. This link is ought to be shared, hence the
/// name.
pub struct ShareLink {
    adjectives: &'static [&'static str; 100],
    animals: &'static [&'static str; 100],
}

const ADJECTIVES: &[&str; 100] = &[
    "Angry", "Bold", "Brave", "Calm", "Clever", "Crazy", "Dark", "Deep", "Eager", "Fancy", "Fast",
    "Fierce", "Fine", "Fresh", "Friendly", "Funny", "Gentle", "Gloomy", "Grand", "Great", "Happy",
    "Hard", "Harsh", "Heavy", "High", "Honest", "Hot", "Huge", "Humble", "Hungry", "Ideal",
    "Innocent", "Jolly", "Juicy", "Kind", "Large", "Lazy", "Light", "Lonely", "Loud", "Lovely",
    "Lucky", "Lush", "Mad", "Mean", "Messy", "Mighty", "Modern", "Narrow", "Neat", "Nervous",
    "Nice", "Noble", "Nosy", "Odd", "Old", "Open", "Perfect", "Plain", "Pleasant", "Polite",
    "Powerful", "Proud", "Quick", "Quiet", "Rare", "Real", "Rich", "Rude", "Sad", "Safe", "Salty",
    "Scary", "Serious", "Sharp", "Shiny", "Short", "Shy", "Silly", "Simple", "Sleepy", "Slow",
    "Small", "Smart", "Snappy", "Soft", "Sour", "Special", "Speedy", "Spicy", "Strange", "Strong",
    "Sweet", "Tall", "Tiny", "Tough", "Tricky", "Ugly", "Wild", "Witty",
];

const ANIMALS: &[&str; 100] = &[
    "Dog",
    "Cat",
    "Horse",
    "Cow",
    "Pig",
    "Sheep",
    "Goat",
    "Lion",
    "Tiger",
    "Bear",
    "Wolf",
    "Fox",
    "Deer",
    "Moose",
    "Elk",
    "Rabbit",
    "Hare",
    "Squirrel",
    "Mouse",
    "Rat",
    "Bat",
    "Elephant",
    "Giraffe",
    "Zebra",
    "Rhino",
    "Hippo",
    "Cheetah",
    "Leopard",
    "Panther",
    "Jaguar",
    "Kangaroo",
    "Koala",
    "Sloth",
    "Armadillo",
    "Porcupine",
    "Hedgehog",
    "Raccoon",
    "Otter",
    "Ferret",
    "Skunk",
    "Chimpanzee",
    "Gorilla",
    "Monkey",
    "Baboon",
    "Orangutan",
    "Dolphin",
    "Whale",
    "Shark",
    "Octopus",
    "Squid",
    "Jellyfish",
    "Seahorse",
    "Starfish",
    "Crab",
    "Lobster",
    "Shrimp",
    "Clam",
    "Snail",
    "Tortoise",
    "Turtle",
    "Crocodile",
    "Alligator",
    "Lizard",
    "Snake",
    "Frog",
    "Toad",
    "Eagle",
    "Hawk",
    "Falcon",
    "Owl",
    "Vulture",
    "Parrot",
    "Penguin",
    "Swan",
    "Duck",
    "Goose",
    "Turkey",
    "Rooster",
    "Chicken",
    "Pigeon",
    "Peacock",
    "Sparrow",
    "Hummingbird",
    "Woodpecker",
    "Magpie",
    "Raven",
    "Crow",
    "Stork",
    "Flamingo",
    "Cormorant",
    "Antelope",
    "Buffalo",
    "Bison",
    "Yak",
    "Hyena",
    "Meerkat",
    "Platypus",
    "Wombat",
    "Stingray",
    "Mongoose",
];

impl ShareLink {
    /// Constructs a new one
    pub fn new() -> ShareLink {
        Self {
            adjectives: ADJECTIVES,
            animals: ANIMALS,
        }
    }

    /// Genereates a link consisting of an adjective, an animal and a four digit random number. An
    /// example would be 'SillyGoose1234'.
    pub fn generate_link(&self) -> String {
        let mut link = String::from("");
        let mut rng = rand::rngs::OsRng;
        let random_num = rng.gen_range(0..10000);
        link.push_str(self.adjectives.choose(&mut rng).unwrap());
        link.push_str(self.animals.choose(&mut rng).unwrap());
        link.push_str(format!("{:0>4}", random_num.to_string()).as_str());

        return link;
    }
}
