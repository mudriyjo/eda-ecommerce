use std::collections::HashMap;

#[derive(Debug)]
pub struct Env {
    pub(crate) server_address: String,
    pub(crate) db_connection_string: String,
    pub(crate) kafka_group_id: String,
    pub(crate) kafka_broker: String,
}

impl Env {
    fn list_of_variables() -> HashMap<String, Option<String>> {
        vec![
            ("AUTH_SERVER".to_string(), None),
            ("AUTH_DATABASE_URL".to_string(), None),
            ("KAFKA_GROUP_ID".to_string(), None),
            ("KAFKA_BROKER".to_string(), None),
        ].into_iter().collect()
    }
    
    fn not_all_variables_exist(env_variables: &HashMap<String, Option<String>>) -> bool {
        env_variables.iter().any(|entry| entry.1.is_none())
    }

    fn read_all_variables(env_variables: &mut HashMap<String, Option<String>>) {
        for entry in env_variables.iter_mut() {
            if let Ok(variable) = std::env::var(entry.0) {
                *entry.1 = Some(variable)
            }
        }
    }

    pub fn new() -> anyhow::Result<Self> {
        let mut variables = Self::list_of_variables();
        
        // Read from env 
        Self::read_all_variables(&mut variables);

        if Self::not_all_variables_exist(&variables) {
            dotenv::dotenv().expect("Can't find .env file or variables and can't load them");
        }
        
        // Read after read from .env file
        Self::read_all_variables(&mut variables);

        if Self::not_all_variables_exist(&variables) {
            let var_string = variables.iter_mut().fold("".to_string(), |mut acc, el| {
                acc.push_str(format!("{} : {:?} \n", el.0, el.1).as_str());
                acc
            });
            anyhow::bail!(format!("Not all variables exist: {}", var_string))
        } else {
            Ok(Env{
                server_address: variables.get("SERVER").unwrap().clone().unwrap(),
                db_connection_string: variables.get("DATABASE_URL").unwrap().clone().unwrap(),
                kafka_group_id: variables.get("KAFKA_GROUP_ID").unwrap().clone().unwrap(),
                kafka_broker: variables.get("KAFKA_BROKER").unwrap().clone().unwrap()
            })
        }
    }
}