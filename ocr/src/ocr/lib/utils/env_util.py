import os
import yaml
import logging

YELLOW = "\033[33m"
RESET = "\033[0m"

class EnvVar:
    def __init__(self, env_name, env_value):
        self.env_name = env_name
        self.env_value = env_value

    def as_string(self):
        return self.env_value

    def as_bool(self):
        if self.env_value.lower() in ['true', 'false']:
            return self.env_value.lower() == 'true'
        else:
            logging.error(f"Invalid value for: {self.env_name}. Must be true or false.")
            raise ValueError(f"Invalid value for: {self.env_name}. Must be true or false.")

class Environment:
    def __init__(self, config_file_path, config_extension="", allow_config_file_nonexistence=True, check_env_var=True, nullable=False):
        self.config_file_path = config_file_path
        self.config_extension = config_extension
        self.allow_config_file_nonexistence = allow_config_file_nonexistence
        self.check_env_var = check_env_var
        self.nullable = nullable

        if not os.path.exists(config_file_path) and not allow_config_file_nonexistence:
            logging.error(f"Config file does not exist: {config_file_path}")
            raise FileNotFoundError(f"Config file does not exist: {config_file_path}")

        if not self.config_extension:
            self.config_extension = os.path.splitext(config_file_path)[1]
            if not self.config_extension:
                logging.error(f"Cannot determine config file extension for {config_file_path}")
                raise ValueError(f"Cannot determine config file extension for {config_file_path}")

        self.config = self.load_config()

    def load_config(self):
        try:
            with open(self.config_file_path, 'r') as config_file:
                if self.config_extension in ['.yaml', '.yml']:
                    return yaml.safe_load(config_file)
                else:
                    logging.error(f"Unsupported config file extension: {self.config_extension}")
                    raise ValueError(f"Unsupported config file extension: {self.config_extension}")
        except Exception as e:
            logging.warning(f"{self.highlight_warning(f'Error reading config file: {e}')}")
            if not self.allow_config_file_nonexistence:
                raise e
            return {}

    def get_env_config(self, key):
        return self.config.get(key, "")

    def get_env(self, key, default_value=""):
        env_value = self.get_env_config(key)

        if env_value:
            return EnvVar(key, env_value)

        if self.check_env_var:
            env_value = os.getenv(key, "")
            if env_value:
                return EnvVar(key, env_value)
            else:
                logging.warning(self.highlight_warning(f"Env var {key} not set"))

        if default_value:
            return EnvVar(key, default_value)

        if self.nullable:
            logging.warning(self.highlight_warning(f"Default value for env var {key} not set but nullable values allowed. Returning empty string"))
            return EnvVar(key, "")

        logging.error(f"Env var {key} not set. Default value is undefined")
        raise EnvironmentError(f"Env var {key} not set. Default value is undefined")

    @staticmethod
    def highlight_warning(warning):
        return f"{YELLOW}{warning}{RESET}"
