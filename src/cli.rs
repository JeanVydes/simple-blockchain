use crate::models;

pub fn init_cli_processor() -> models::CLIConfiguration {
    let args: Vec<String> = std::env::args().collect();
    let mut configuration = models::CLIConfiguration {
        port: models::NODE_DEFAULT_PORT,
        host: models::NODE_DEFAULT_ADDRESS.to_string(),
        workdir: models::NODE_DEFAULT_DIR_DATA.to_string(),
        log: false,
    };

    for (i, argument) in args.iter().enumerate() {
        if i == 0 {
            continue;
        }

        match argument.as_str() {
            "--port" => {
                if i + 1 < args.len() {
                    let port = args[i + 1].parse::<u16>().unwrap();
                    configuration.port = port;
                }
            }
            "--host" => {
                if i + 1 < args.len() {
                    configuration.host = args[i + 1].clone();
                }
            }
            "--workdir" => {
                if i + 1 < args.len() {
                    configuration.workdir = args[i + 1].clone();
                }
            }
            "--debug" => {
                configuration.log = true;
            }
            _ => {}
        }
    }

    return configuration;
}