use yaml_rust::{YamlLoader, Yaml, yaml};
use std::fs::File;
use std::io::Read;
use std::env;

pub fn get_selected_plugin_list() -> Vec<String> {
    let plugins_str = env::var("PLUGINS")
	.expect("\n\n\
		 ┌─────────────────────────────────┐\n\
		 │               BUILD FAILED                │\n\
		 │PLUGINS environment variable not found.    │\n\
		 │Help: You should call make instead of cargo│\n\
		 └─────────────────────────────────┘\
		 \n\n");
    if plugins_str.len() > 0 {
	plugins_str
	    .split(" ").map(|s| s.to_string()).collect()
    } else {
	Vec::new()
    }
}

pub fn get_yaml(file: &str) -> Yaml {
    let mut base = File::open(file).expect(file);
    let mut yaml_str = String::new();
    if let Err(err) = base.read_to_string(&mut yaml_str) {
	panic!("Could not read yaml base file {}", err);	
    }
    yaml_str = yaml_str.replace("$VERSION", &env!("VERSION"));
    YamlLoader::load_from_str(&yaml_str).unwrap().swap_remove(0)
}



#[allow(unused)]
pub fn get_yaml_top_level<'a>(yaml: &'a mut Yaml, fieldsearch: &str) -> Option<&'a mut String> {
    match yaml {
	Yaml::Hash(hash) => {
	    for field in hash {
		if let Yaml::String(fieldname) = field.0 {
		    if fieldname == fieldsearch {
			match field.1 {
			    Yaml::String(arr) => {
				return Some(arr);
			    },
			    _ => panic!("Incorrect arg format on cli_base"),
			}
		    }
		}
	    }
	},
	_ => panic!("Incorrect yaml format on cli_base"),
    }
    None
}

#[allow(unused)]
pub fn get_yaml_args(yaml: &mut Yaml) -> &mut Vec<yaml::Yaml> {
    match yaml {
	Yaml::Hash(hash) => {
	    for field in hash {
		if let Yaml::String(fieldname) = field.0 {
		    if fieldname == "args" {
			match field.1 {
			    Yaml::Array(arr) => {
				sanitize_args(arr);
				return arr;
			    },
			    _ => panic!("Incorrect arg format on cli_base"),
			}
		    }
		}
	    }
	},
	_ => panic!("Incorrect yaml format on cli_base"),
    }
    panic!("No args found in yaml object");
}

fn sanitize_args(args: &mut Vec<yaml::Yaml>) {
    *args = args.drain(..).map(|yml| {
	if let Yaml::Hash(mut hash) = yml {
	    for (_, arg) in hash.iter_mut() {
		if let Yaml::Hash(ref mut properties) = arg {
		    let name_visible_aliases       = Yaml::String("visible_aliases"      .to_owned());
		    let name_visible_short_aliases = Yaml::String("visible_short_aliases".to_owned());
		    let visible_aliases       = properties.remove(&name_visible_aliases);
		    let visible_short_aliases = properties.remove(&name_visible_short_aliases);

		    let mut alias_help = Vec::new();

		    if let Some(Yaml::String(visible_aliases)) = visible_aliases {
			let name_aliases = Yaml::String("aliases".to_owned());
			let aliases      = properties.remove(&name_aliases);
			let mut new_aliases = visible_aliases;
			if let Some(Yaml::String(aliases)) = aliases {
			    new_aliases.push(' ');
			    new_aliases.push_str(&aliases);
			};
			for alias in new_aliases.split(' ') {
			    alias_help.push(format!("--{}", alias));
			}
			properties.insert(name_aliases, Yaml::String(new_aliases));
		    }

		    if let Some(Yaml::String(visible_short_aliases)) = visible_short_aliases {
			let name_short_aliases = Yaml::String("short_aliases"        .to_owned());
			let short_aliases      = properties.remove(&name_short_aliases);
			let mut new_short_aliases = visible_short_aliases;
			if let Some(Yaml::String(short_aliases)) = short_aliases {
			    new_short_aliases.push(' ');
			    new_short_aliases.push_str(&short_aliases);
			};
			for alias in new_short_aliases.split(' ') {
			    alias_help.push(format!("-{}", alias));
			}
			properties.insert(name_short_aliases, Yaml::String(new_short_aliases));
		    }

		    if !alias_help.is_empty() {
			let alias_string = format!("\n [aliases: {}]", alias_help.join(", "));
			if let Some(Yaml::String(ref mut long_help)) = properties.get_mut(&Yaml::String("long_help".to_owned())).as_mut() {
			    long_help.push_str(&alias_string);
			} else if let Some(Yaml::String(ref mut help)) = properties.get_mut(&Yaml::String("help".to_owned())).as_mut() {
			    help.push_str(&alias_string);
			}
		    }
		}
	    }
	    return Yaml::Hash(hash);
	}
	yml
    }).collect();
}
