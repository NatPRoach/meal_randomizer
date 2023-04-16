use clap::{builder::PossibleValue, Parser, ValueEnum};
use log::debug;
use rand::{seq::IteratorRandom, thread_rng};
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeasonFilter {
    Any,
    Winter,
    Spring,
    Summer,
    Fall,
}

impl Default for SeasonFilter {
    fn default() -> Self {
        Self::Any
    }
}

impl std::fmt::Display for SeasonFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

impl ValueEnum for SeasonFilter {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::Any,
            Self::Winter,
            Self::Spring,
            Self::Summer,
            Self::Fall,
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            Self::Any => PossibleValue::new("any").help("Do not filter by season"),
            Self::Winter => PossibleValue::new("winter").help("Only provide winter recipes"),
            Self::Spring => PossibleValue::new("spring").help("Only provide spring recipes"),
            Self::Summer => PossibleValue::new("summer").help("Only provide summer recipes"),
            Self::Fall => PossibleValue::new("fall").help("Only provide fall recipes"),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EthnicityFilter {
    Any,
    American,
    Chinese,
    EasternEuropean,
    Ethiopian,
    French,
    Indian,
    Japanese,
    Mediteranean,
    Mexican,
    Spanish,
}

impl Default for EthnicityFilter {
    fn default() -> Self {
        Self::Any
    }
}

impl std::fmt::Display for EthnicityFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

impl ValueEnum for EthnicityFilter {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::Any,
            Self::American,
            Self::Chinese,
            Self::EasternEuropean,
            Self::Ethiopian,
            Self::French,
            Self::Indian,
            Self::Japanese,
            Self::Mediteranean,
            Self::Mexican,
            Self::Spanish,
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            Self::Any => PossibleValue::new("any").help("Do not filter by ethnicity."),
            Self::American => {
                PossibleValue::new("american").help("Filter for American style meals.")
            }
            Self::French => PossibleValue::new("french").help("Filter for French style meals."),
            Self::Spanish => PossibleValue::new("spanish").help("Filter for Spanish style meals."),
            Self::Ethiopian => {
                PossibleValue::new("ethiopian").help("Filter for Ethiopian style meals.")
            }
            Self::Chinese => PossibleValue::new("chinese").help("Filter for Chinese style meals."),
            Self::Japanese => {
                PossibleValue::new("japanese").help("Filter for Japanese style meals.")
            }
            Self::EasternEuropean => PossibleValue::new("eastern-european")
                .help("Filter for Eastern European style meals."),
            Self::Mediteranean => {
                PossibleValue::new("mediteranean").help("Filter for Mediteranean style meals.")
            }
            Self::Indian => PossibleValue::new("indian").help("Filter for Indian style meals."),
            Self::Mexican => PossibleValue::new("mexican").help("Filter for Mexican style meals."),
        })
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct Recipe {
    name: String,
    seasons: Vec<SeasonFilter>,
    ethnicities: Vec<EthnicityFilter>,
    ingredients: Vec<String>, // TODO - replace with an ingredient struct.
    steps: Vec<String>,
}

struct Recipes {
    inner: HashMap<PathBuf, Recipe>,
    season_filter: HashSet<SeasonFilter>,
    ethnicity_filter: HashSet<EthnicityFilter>,
}

impl Recipes {
    pub fn from_args(args: &GetRandomRecipes) -> Self {
        let mut inner = HashMap::new();

        for input_path_res in fs::read_dir(&args.recipes_dir).unwrap() {
            let input_path = input_path_res.unwrap().path();
            if input_path.ends_with(".yaml") || input_path.ends_with(".yml") {
                let reader = BufReader::new(File::open(&input_path).unwrap());
                inner.insert(input_path, serde_yaml::from_reader(reader).unwrap());
            }
        }

        Self {
            inner,
            season_filter: args.season.iter().cloned().collect::<HashSet<_>>(),
            ethnicity_filter: args.ethnicity.iter().cloned().collect::<HashSet<_>>(),
        }
    }

    pub fn passes_filters(&self, recipe: &Recipe) -> bool {
        let passes_ethnicity_filter = self.ethnicity_filter.contains(&EthnicityFilter::Any)
            || recipe
                .ethnicities
                .iter()
                .any(|e| self.ethnicity_filter.contains(e));
        let passes_season_filter = self.season_filter.contains(&SeasonFilter::Any)
            || recipe
                .seasons
                .iter()
                .any(|e| self.season_filter.contains(e));
        passes_ethnicity_filter && passes_season_filter
    }

    pub fn randomly_select_recipes(&self, num_recipes: usize) -> Vec<PathBuf> {
        let keys = self
            .inner
            .keys()
            .map(|p| (p, self.inner.get(p).unwrap()))
            .filter(|(_p, r)| self.passes_filters(r))
            .collect::<Vec<_>>();
        let mut rng = thread_rng();

        let num_to_select = if keys.len() < num_recipes {
            debug!("Number of recipes matching filter was less than requested number of recipes, returning all recipes available matching filters.");
            keys.len()
        } else {
            num_recipes
        };

        keys.iter()
            .choose_multiple(&mut rng, num_to_select)
            .iter()
            .map(|(p, _r)| (*p).clone())
            .collect::<Vec<_>>()
    }
}

#[derive(Parser, Debug)]
struct GetRandomRecipes {
    /// The seasonal recipe types to filter to.
    #[clap(short, long, num_args = 1..)]
    season: Vec<SeasonFilter>,

    /// The ethnicities of recipe types to filter to.
    #[clap(short, long, num_args = 1..)]
    ethnicity: Vec<EthnicityFilter>,

    /// The directory containing the recipes to randomize over in YAML format
    #[clap(short, long)]
    recipes_dir: PathBuf,

    /// The number of recipes to return
    #[clap(short, long, default_value = "3")]
    num_recipes: usize,
}

fn main() {
    let args = GetRandomRecipes::parse();
    let recipes = Recipes::from_args(&args);
    let selected_recipe_paths = recipes.randomly_select_recipes(args.num_recipes);

    // TODO - nice PDF grocery list
    // TODO - nice PDF recipe
    println!("{selected_recipe_paths:?}")
}
