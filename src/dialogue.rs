use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;

// Character types based on sprites in rogues.png
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CharacterType {
    Dwarf,
    Elf,
    Ranger,
    Rogue,
    Bandit,
    Knight,
    Fighter,
    FemaleKnight,
    ShieldKnight,
    Monk,
    Priest,
    WarCleric,
    Templar,
    Barbarian,
    Swordsman,
    Fencer,
    Wizard,
    Druid,
    Sage,
    DwarfMage,
    Warlock,
    Farmer,
    Baker,
    Blacksmith,
    Scholar,
    Peasant,
    Shopkeeper,
    Elder,
    Generic,
}

impl CharacterType {
    // Convert sprite name to character type
    pub fn from_sprite_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "dwarf" => CharacterType::Dwarf,
            "elf" => CharacterType::Elf,
            "ranger" => CharacterType::Ranger,
            "rogue" => CharacterType::Rogue,
            "bandit" => CharacterType::Bandit,
            "knight" | "male knight" => CharacterType::Knight,
            "male fighter" => CharacterType::Fighter,
            "female knight" | "female knight (helmetless)" => CharacterType::FemaleKnight,
            "shield knight" => CharacterType::ShieldKnight,
            "monk" => CharacterType::Monk,
            "priest" => CharacterType::Priest,
            "female war cleric" | "male war cleric" => CharacterType::WarCleric,
            "templar" => CharacterType::Templar,
            "male barbarian" | "female barbarian" | "male winter barbarian" | "female winter barbarian" => CharacterType::Barbarian,
            "swordsman" => CharacterType::Swordsman,
            "fencer" => CharacterType::Fencer,
            "female wizard" | "male wizard" => CharacterType::Wizard,
            "druid" => CharacterType::Druid,
            "desert sage" => CharacterType::Sage,
            "dwarf mage" => CharacterType::DwarfMage,
            "warlock" => CharacterType::Warlock,
            "farmer (wheat thresher)" | "farmer (scythe)" | "farmer (pitchfork)" => CharacterType::Farmer,
            "baker" => CharacterType::Baker,
            "blacksmith" => CharacterType::Blacksmith,
            "scholar" => CharacterType::Scholar,
            "peasant" | "peasant / coalburner" => CharacterType::Peasant,
            "shopkeep" => CharacterType::Shopkeeper,
            "elderly woman" | "elderly man" => CharacterType::Elder,
            _ => CharacterType::Generic,
        }
    }

    // Get a name appropriate for this character type
    pub fn generate_name(&self) -> String {
        let mut rng = rand::thread_rng();
        
        match self {
            CharacterType::Dwarf => {
                let first_names = ["Thorin", "Gimli", "Balin", "Dwalin", "Gloin", "Oin", "Bombur", "Bifur", "Bofur", "Durin", "Thrain", "Thror"];
                let last_names = ["Ironfoot", "Stonehelm", "Oakenshield", "Strongarm", "Deepdelver", "Fireforge", "Goldhand", "Anvilbreaker"];
                format!("{} {}", first_names.choose(&mut rng).unwrap(), last_names.choose(&mut rng).unwrap())
            },
            CharacterType::Elf => {
                let first_names = ["Legolas", "Elrond", "Galadriel", "Arwen", "Thranduil", "Celeborn", "Haldir", "Tauriel", "Finrod", "Luthien"];
                let last_names = ["Greenleaf", "Starlight", "Moonwhisper", "Silverbranch", "Nightshade", "Dawnbreaker", "Swiftarrow"];
                format!("{} {}", first_names.choose(&mut rng).unwrap(), last_names.choose(&mut rng).unwrap())
            },
            CharacterType::Ranger => {
                let first_names = ["Aragorn", "Faramir", "Boromir", "Arathorn", "Halbarad", "Strider", "Denethor", "Beregond"];
                let last_names = ["Strider", "Pathfinder", "Wayfarer", "Longstride", "Nightwalker", "Shadowtracker"];
                format!("{} {}", first_names.choose(&mut rng).unwrap(), last_names.choose(&mut rng).unwrap())
            },
            CharacterType::Wizard => {
                let names = ["Gandalf", "Saruman", "Radagast", "Alatar", "Pallando", "Merlin", "Elminster", "Mordenkainen", "Tenser", "Bigby", "Otiluke"];
                let titles = ["the Grey", "the White", "the Brown", "the Blue", "the Wise", "the Arcane", "the Magnificent", "the Mysterious"];
                format!("{} {}", names.choose(&mut rng).unwrap(), titles.choose(&mut rng).unwrap())
            },
            CharacterType::Barbarian => {
                let names = ["Conan", "Krom", "Thulsa", "Brak", "Grommash", "Thorg", "Ragnar", "Bjorn", "Leif", "Olaf", "Ulfric"];
                let titles = ["the Destroyer", "the Mighty", "Bloodaxe", "Skullcrusher", "Ironhide", "Stormbringer", "Thunderfist"];
                format!("{} {}", names.choose(&mut rng).unwrap(), titles.choose(&mut rng).unwrap())
            },
            CharacterType::Knight | CharacterType::FemaleKnight | CharacterType::ShieldKnight => {
                let first_names = ["Lancelot", "Gawain", "Percival", "Galahad", "Arthur", "Bedivere", "Kay", "Bors", "Tristan", "Gareth"];
                let titles = ["the Brave", "the Bold", "the Valiant", "the Steadfast", "the Loyal", "the Just", "the Honorable"];
                format!("Sir {} {}", first_names.choose(&mut rng).unwrap(), titles.choose(&mut rng).unwrap())
            },
            CharacterType::Priest | CharacterType::WarCleric | CharacterType::Templar => {
                let titles = ["Brother", "Sister", "Father", "Mother", "Chaplain", "Cleric", "Reverend"];
                let names = ["Thomas", "Benedict", "Augustine", "Ambrose", "Gregory", "Jerome", "Hildegard", "Teresa", "Catherine", "Cecilia"];
                format!("{} {}", titles.choose(&mut rng).unwrap(), names.choose(&mut rng).unwrap())
            },
            CharacterType::Shopkeeper => {
                let first_names = ["Olaf", "Greta", "Hans", "Helga", "Otto", "Brunhilde", "Gustav", "Ingrid"];
                let last_names = ["Merchant", "Seller", "Trader", "Vendor", "Shopkeep", "Storeowner", "Purveyor"];
                format!("{} the {}", first_names.choose(&mut rng).unwrap(), last_names.choose(&mut rng).unwrap())
            },
            CharacterType::Blacksmith => {
                let first_names = ["Hephaestus", "Vulcan", "Wayland", "Goibniu", "Ilmarinen", "Svarog", "Tvastar"];
                let titles = ["the Smith", "Ironhand", "Steelforger", "Hammerfall", "Anvilsong", "Flamebeard"];
                format!("{} {}", first_names.choose(&mut rng).unwrap(), titles.choose(&mut rng).unwrap())
            },
            _ => {
                // Generic names for other types
                let first_names = ["John", "Mary", "Robert", "Patricia", "James", "Jennifer", "Michael", "Linda", "William", "Elizabeth"];
                let last_names = ["Smith", "Johnson", "Williams", "Jones", "Brown", "Davis", "Miller", "Wilson", "Moore", "Taylor"];
                format!("{} {}", first_names.choose(&mut rng).unwrap(), last_names.choose(&mut rng).unwrap())
            }
        }
    }
}

// Generate dialogue based on character type
pub fn generate_dialogue(character_type: &CharacterType) -> Vec<String> {
    let mut rng = rand::thread_rng();
    let mut dialogue = Vec::new();
    
    // Common greetings that any character might say
    let common_greetings = [
        "Hello there, traveler.",
        "Greetings, adventurer.",
        "Well met, stranger.",
        "Ah, a visitor. How unusual.",
        "Welcome to these parts.",
        "I don't see many travelers here.",
        "Stay a while and listen.",
        "What brings you to these dangerous caves?",
        "Be careful in these parts.",
        "Watch your step around here.",
    ];
    
    // Add 1-2 common greetings
    let num_greetings = rng.gen_range(1..=2);
    for _ in 0..num_greetings {
        if let Some(greeting) = common_greetings.choose(&mut rng) {
            dialogue.push(greeting.to_string());
        }
    }
    
    // Character-specific dialogue
    match character_type {
        CharacterType::Dwarf => {
            let dwarf_lines = [
                "These caves remind me of the mines of my homeland.",
                "I've been mapping these tunnels for years.",
                "There's gold in these hills, I can smell it!",
                "Watch for loose rocks overhead. These tunnels aren't all stable.",
                "My beard has grown three inches since I started exploring here.",
                "Nothing beats dwarven craftsmanship, you know.",
                "I once found a vein of mithril down here... never could find it again.",
                "The deeper you go, the more dangerous it gets.",
            ];
            add_random_lines(&mut dialogue, &dwarf_lines, 2, &mut rng);
        },
        CharacterType::Elf => {
            let elf_lines = [
                "I sense ancient magic in these caverns.",
                "The stars guided me here, though I cannot see them underground.",
                "I've lived for centuries, but these caves still hold mysteries for me.",
                "My people rarely venture underground, but necessity drives us all to strange places.",
                "The trees above whisper warnings about what lies below.",
                "I'm studying the unique fungi that grow only in these caves.",
                "Even in darkness, an elf can find beauty.",
                "My eyes see farther in the dark than most.",
            ];
            add_random_lines(&mut dialogue, &elf_lines, 2, &mut rng);
        },
        CharacterType::Wizard | CharacterType::DwarfMage | CharacterType::Warlock => {
            let wizard_lines = [
                "The magical energies here are... unusual. Most fascinating.",
                "I'm conducting research on the arcane properties of these caverns.",
                "Don't touch anything glowing. Trust me on this.",
                "I've been experimenting with a new spell. Care to see?",
                "There are ancient runes carved into some of these walls. They speak of terrible things.",
                "The boundary between planes is thin in places like this.",
                "I sense a powerful artifact somewhere below us.",
                "Magic behaves strangely in these depths. Be cautious with any enchanted items.",
            ];
            add_random_lines(&mut dialogue, &wizard_lines, 2, &mut rng);
        },
        CharacterType::Knight | CharacterType::FemaleKnight | CharacterType::ShieldKnight | CharacterType::Fighter => {
            let knight_lines = [
                "I've sworn an oath to protect travelers in these dangerous parts.",
                "My blade has tasted the blood of many monsters that lurk here.",
                "Honor and courage will see you through the darkest passages.",
                "I seek a worthy opponent to test my skills against.",
                "These ruins once belonged to a great kingdom. Now look at them.",
                "I'm on a quest for my liege. I cannot say more.",
                "Stand behind me if we encounter danger. My shield has never failed.",
                "The code of chivalry guides me, even in this forsaken place.",
            ];
            add_random_lines(&mut dialogue, &knight_lines, 2, &mut rng);
        },
        CharacterType::Priest | CharacterType::WarCleric | CharacterType::Templar | CharacterType::Monk => {
            let religious_lines = [
                "May the light guide your path through this darkness.",
                "I'm here to cleanse these caverns of unholy influences.",
                "Evil lurks in the shadows. Stay vigilant.",
                "I've been blessed with divine protection. Stay close.",
                "These caves were once a sacred site, before the corruption spread.",
                "I'm searching for a lost relic of my faith.",
                "Prayer strengthens the spirit, especially in places like this.",
                "The gods watch over us, even here beneath the earth.",
            ];
            add_random_lines(&mut dialogue, &religious_lines, 2, &mut rng);
        },
        CharacterType::Rogue | CharacterType::Bandit => {
            let rogue_lines = [
                "Keep your voice down. You never know who's listening.",
                "I know all the best hiding spots down here.",
                "There's treasure to be found, if you know where to look.",
                "I'm not hiding from the law, I'm just... taking a break from society.",
                "Watch your coinpurse. Not everyone down here is as honest as me.",
                "I could tell you what I'm really doing here, but then I'd have to kill you.",
                "The shadows are a rogue's best friend.",
                "Quick fingers and quicker wits keep you alive in this business.",
            ];
            add_random_lines(&mut dialogue, &rogue_lines, 2, &mut rng);
        },
        CharacterType::Barbarian | CharacterType::Swordsman => {
            let warrior_lines = [
                "I seek worthy foes to test my strength against!",
                "These caves echo with the screams of those who challenged me.",
                "My blade thirsts for battle!",
                "In my homeland, we hunt monsters like those that lurk here for sport.",
                "Strength and steel are all you need to survive.",
                "I've slain beasts twice your size with my bare hands.",
                "The weak perish, the strong survive. That is the law of these caves.",
                "I came seeking glory and adventure. I found plenty of both.",
            ];
            add_random_lines(&mut dialogue, &warrior_lines, 2, &mut rng);
        },
        CharacterType::Shopkeeper => {
            let merchant_lines = [
                "Interested in buying some supplies? I've got the best prices around.",
                "Business is slow down here, but the profit margins make up for it.",
                "I accept gold, silver, and interesting artifacts as payment.",
                "Everything's for sale, for the right price.",
                "I've got items you won't find on the surface.",
                "Be careful with that! You break it, you buy it.",
                "I trade with all the local denizens. Even the ones you'd rather avoid.",
                "Need something specific? I might be able to procure it... for a fee.",
            ];
            add_random_lines(&mut dialogue, &merchant_lines, 2, &mut rng);
        },
        CharacterType::Blacksmith => {
            let smith_lines = [
                "The ore found in these caves makes for exceptional weapons.",
                "I can repair your equipment if you need it. For a price, of course.",
                "A good blade is the difference between life and death down here.",
                "I've been forging for forty years. Nobody makes them better.",
                "The heat of the forge keeps the cave creatures at bay.",
                "I'm experimenting with some unusual metals I found deeper in.",
                "A warrior is only as good as their weapon. Remember that.",
                "The rhythmic sound of hammering helps me forget I'm underground.",
            ];
            add_random_lines(&mut dialogue, &smith_lines, 2, &mut rng);
        },
        CharacterType::Scholar => {
            let scholar_lines = [
                "I'm documenting the unique ecosystem of these caverns.",
                "The historical significance of these ruins cannot be overstated.",
                "My research suggests this area was once part of an ancient civilization.",
                "The inscriptions on these walls tell a fascinating story.",
                "I've been cataloging the various fungi species. Quite remarkable diversity.",
                "Knowledge is the true treasure, my friend.",
                "I've filled three journals already, and I've barely scratched the surface.",
                "The academic community scoffed at my theories. They won't be laughing when I return with proof.",
            ];
            add_random_lines(&mut dialogue, &scholar_lines, 2, &mut rng);
        },
        _ => {
            // Generic dialogue for other types
            let generic_lines = [
                "I've been exploring these caves for some time now.",
                "There are strange noises coming from the deeper levels.",
                "I'm just trying to survive down here, same as everyone.",
                "Have you seen anything unusual in your travels?",
                "The air feels different in these parts. Can you sense it?",
                "I wouldn't go that way if I were you.",
                "Sometimes I think these caves are changing around us.",
                "I've heard rumors of great treasure deeper down.",
                "Trust no one down here. Not even me.",
                "The darkness plays tricks on your mind after a while.",
            ];
            add_random_lines(&mut dialogue, &generic_lines, 2, &mut rng);
        }
    }
    
    // Add a farewell
    let farewells = [
        "Safe travels, friend.",
        "May your path be clear of danger.",
        "Until we meet again.",
        "Watch your back down here.",
        "Remember what I told you.",
        "If you survive, come find me again.",
        "The shadows hide many secrets... and dangers.",
        "Don't forget to rest when you can.",
        "Keep your weapon close and your wits closer.",
        "Farewell, adventurer.",
    ];
    
    if let Some(farewell) = farewells.choose(&mut rng) {
        dialogue.push(farewell.to_string());
    }
    
    dialogue
}

// Helper function to add random lines from a slice to the dialogue vector
fn add_random_lines(dialogue: &mut Vec<String>, lines: &[&str], count: usize, rng: &mut impl rand::Rng) {
    let mut available_lines = lines.to_vec();
    let count = count.min(available_lines.len());
    
    for _ in 0..count {
        if available_lines.is_empty() {
            break;
        }
        
        let index = rng.gen_range(0..available_lines.len());
        dialogue.push(available_lines[index].to_string());
        available_lines.remove(index);
    }
}

// Get all available character sprites from the rogues.txt file
pub fn get_available_character_sprites() -> Vec<String> {
    vec![
        "dwarf".to_string(),
        "elf".to_string(),
        "ranger".to_string(),
        "rogue".to_string(),
        "bandit".to_string(),
        "knight".to_string(),
        "male fighter".to_string(),
        "female knight".to_string(),
        "female knight (helmetless)".to_string(),
        "shield knight".to_string(),
        "monk".to_string(),
        "priest".to_string(),
        "female war cleric".to_string(),
        "male war cleric".to_string(),
        "templar".to_string(),
        "male barbarian".to_string(),
        "male winter barbarian".to_string(),
        "female winter barbarian".to_string(),
        "swordsman".to_string(),
        "fencer".to_string(),
        "female barbarian".to_string(),
        "female wizard".to_string(),
        "male wizard".to_string(),
        "druid".to_string(),
        "desert sage".to_string(),
        "dwarf mage".to_string(),
        "warlock".to_string(),
        "farmer (wheat thresher)".to_string(),
        "farmer (scythe)".to_string(),
        "farmer (pitchfork)".to_string(),
        "baker".to_string(),
        "blacksmith".to_string(),
        "scholar".to_string(),
        "peasant / coalburner".to_string(),
        "peasant".to_string(),
        "shopkeep".to_string(),
        "elderly woman".to_string(),
        "elderly man".to_string(),
    ]
}

// Generate dialogue based on character type and biome
pub fn generate_biome_dialogue(character_type: &CharacterType, biome: &crate::biome::BiomeType) -> String {
    let mut rng = rand::thread_rng();
    
    // Common biome-specific lines that any character might say
    let biome_lines = match biome {
        crate::biome::BiomeType::Caves => {
            vec![
                "These caves seem to go on forever.",
                "Watch your step, the ground is slippery here.",
                "I've heard strange noises echoing from deeper in these caves.",
                "The air is damp and cold in these caverns.",
                "These caves hold many secrets for those brave enough to explore them.",
                "I've been mapping these tunnels for weeks now.",
                "The minerals in these cave walls shimmer beautifully in the light.",
                "Stay alert - cave-ins are common in this area.",
            ]
        },
        crate::biome::BiomeType::Groves => {
            vec![
                "The plants here grow despite the lack of sunlight. Fascinating.",
                "These groves are unusually lush for being underground.",
                "The mushrooms here are quite luminescent, aren't they?",
                "I've never seen vegetation like this before.",
                "Something about this place feels... alive.",
                "The air is surprisingly fresh in these underground groves.",
                "These plants have adapted to life without the sun.",
                "Some of these fungi are quite valuable to alchemists.",
            ]
        },
        crate::biome::BiomeType::Labyrinth => {
            vec![
                "Many have gotten lost in these winding passages.",
                "I've been trying to map this labyrinth for days.",
                "They say a terrible beast lurks at the center of this maze.",
                "The builders of this labyrinth were quite clever with their traps.",
                "Follow the markings on the walls if you don't want to get lost.",
                "I've heard people screaming in the distance. Then silence.",
                "The walls seem to shift when no one is looking.",
                "I swear I've passed this exact spot three times already.",
            ]
        },
        crate::biome::BiomeType::Catacombs => {
            vec![
                "The dead rest uneasily in these catacombs.",
                "Show respect here - we walk among the remains of the ancient ones.",
                "I've felt... presences... watching me in these halls.",
                "The inscriptions on these tombs are in a language long forgotten.",
                "Don't disturb the remains if you value your life.",
                "The air is thick with dust and... something else.",
                "These catacombs predate any civilization I know of.",
                "I've heard whispers when no one else is around.",
            ]
        },
    };
    
    // Character-biome specific lines for certain combinations
    let character_biome_specific = match (character_type, biome) {
        (CharacterType::Dwarf, crate::biome::BiomeType::Caves) => Some(vec![
            "These caves remind me of my ancestral home, though not as well-crafted.",
            "I can sense a rich vein of ore nearby. Dwarven intuition never fails.",
            "My people could carve a magnificent hall from these natural formations.",
            "The rock quality here is decent. Good for mining, better for building.",
        ]),
        (CharacterType::Elf, crate::biome::BiomeType::Groves) => Some(vec![
            "Even underground, life finds a way. It reminds me of our forest homes.",
            "I can feel the ancient magic nurturing these plants. It's familiar, yet different.",
            "These fungi sing a different song than the trees above, but beautiful nonetheless.",
            "My people would find this place sacred, despite being beneath the earth.",
        ]),
        (CharacterType::Wizard | CharacterType::DwarfMage | CharacterType::Warlock, crate::biome::BiomeType::Labyrinth) => Some(vec![
            "The magical currents in this labyrinth are... intriguing. Almost intentional.",
            "This maze was designed to confuse more than the mind. It disrupts magical senses too.",
            "I've been studying the arcane symbols at each junction. They tell a story.",
            "With the right spell, we could see the labyrinth from above. Sadly, I lack the components.",
        ]),
        (CharacterType::Priest | CharacterType::WarCleric | CharacterType::Templar, crate::biome::BiomeType::Catacombs) => Some(vec![
            "I must perform rites to ensure these souls rest peacefully.",
            "The sanctity of death has been disturbed here. I sense it.",
            "These catacombs hold the remains of both the faithful and the heretical.",
            "My order has records of these burial chambers. They are ancient and holy.",
        ]),
        _ => None,
    };
    
    // 30% chance to use character-biome specific line if available
    if let Some(specific_lines) = character_biome_specific {
        if rng.gen_bool(0.3) {
            return specific_lines[rng.gen_range(0..specific_lines.len())].to_string();
        }
    }
    
    // Otherwise use general biome line
    biome_lines[rng.gen_range(0..biome_lines.len())].to_string()
}

// Generate cryptic dialogue that's short and esoteric
pub fn generate_cryptic_dialogue() -> Vec<String> {
    let mut rng = rand::thread_rng();
    let cryptic_lines = [
        "The void whispers...",
        "Shadows dance when unwatched.",
        "Below lies truth.",
        "They come from walls.",
        "Listen to the stones.",
        "Time bends here.",
        "The path changes.",
        "Eyes in darkness.",
        "Ancient ones stir.",
        "Patterns in chaos.",
        "Descent reveals.",
        "Echoes of before.",
        "Walls have memory.",
        "The deep knows.",
        "Cycles return.",
        "Light betrays.",
        "Silence speaks volumes.",
        "Between worlds now.",
        "Not alone here.",
        "Secrets beneath secrets.",
        "The way shifts.",
        "Forgotten knowledge waits.",
        "Dreams become real.",
        "Follow the signs.",
        "Beware the depths.",
        "Reflections lie.",
        "Doors without keys.",
        "The abyss gazes back.",
        "Patterns repeat.",
        "Whispers guide.",
    ];
    
    let mut dialogue = Vec::new();
    let num_lines = rng.gen_range(1..=2);
    
    for _ in 0..num_lines {
        if let Some(line) = cryptic_lines.choose(&mut rng) {
            dialogue.push(line.to_string());
        }
    }
    
    dialogue
}

// Modify the spawn_npc function to use cryptic dialogue
pub fn generate_biome_cryptic_dialogue(biome: &crate::biome::BiomeType) -> String {
    let mut rng = rand::thread_rng();
    
    // Biome-specific cryptic lines
    let biome_lines = match biome {
        crate::biome::BiomeType::Caves => {
            vec![
                "Stones remember footsteps.",
                "Water carves patience.",
                "Darkness breathes here.",
                "Echoes hide meanings.",
                "Walls shift slowly.",
                "Crystal memories glow.",
                "Paths change when unwatched.",
                "The deep has eyes.",
            ]
        },
        crate::biome::BiomeType::Groves => {
            vec![
                "Roots speak secrets.",
                "Light without sun.",
                "Growth from nothing.",
                "Life finds ways.",
                "Green dreams below.",
                "Spores carry thoughts.",
                "Fungi remember.",
                "The garden spreads.",
            ]
        },
        crate::biome::BiomeType::Labyrinth => {
            vec![
                "Paths within paths.",
                "Center ever shifts.",
                "Walls remember ways.",
                "Patterns hide purpose.",
                "The maze watches.",
                "Designed confusion.",
                "No true exit exists.",
                "Follow the marks.",
            ]
        },
        crate::biome::BiomeType::Catacombs => {
            vec![
                "They still whisper.",
                "Death is not silent.",
                "Names forgotten, not gone.",
                "Bones remember flesh.",
                "Ancient sleepers stir.",
                "Dust holds memories.",
                "Tombs without bodies.",
                "The dead walk paths.",
            ]
        },
    };
    
    biome_lines[rng.gen_range(0..biome_lines.len())].to_string()
}
