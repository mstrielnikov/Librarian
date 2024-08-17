package env

import (
	"fmt"
	"github.com/spf13/viper"
	"log"
	"os"
	"path"
	"strconv"
)

const (
	yellow = "\033[33m"
	reset  = "\033[0m"
)

type EnvVar struct {
	envName  string
	envValue string
}

func (v *EnvVar) asString() string {
	return v.envValue
}

func (v *EnvVar) asBool() bool {
	val, err := strconv.ParseBool(v.envValue)
	if err != nil {
		log.Fatalf("Invalid value for : %s. Must be true or false.\n", v.envName)
	}
	return val
}

type Environment struct {
	configFilePath              string
	configExtension             string
	allowConfigFileNonExistanse bool
	checkEnvVar                 bool
	nullable                    bool
}

func NewEnvironment(configFilePath, configExtension string, allowConfigFileNonExistanse, checkEnvVar, nullable bool) *Environment {
	_, err := os.Stat(configFilePath)
	if os.IsNotExist(err) {
		log.Fatalf("Config file does not exist: %s", configFilePath)
	}

	return &Environment{
		configFilePath:              configFilePath,
		configExtension:             configExtension,
		allowConfigFileNonExistanse: allowConfigFileNonExistanse,
		checkEnvVar:                 checkEnvVar,
		nullable:                    nullable,
	}
}

func (e *Environment) envConfigGet(key string) string {
	_, err := os.Stat(e.configFilePath)
	if os.IsNotExist(err) {
		if e.allowConfigFileNonExistanse {
			warning := fmt.Sprintf("config file %s does not exist or not readable. Env vars will be checked instead", e.configFilePath)
			log.Println(highlightWarning(warning))
			return ""
		} else {
			log.Fatalf("config file %s not exists", e.configFilePath)
		}
	}

	if e.configExtension == "" {
		e.configExtension = path.Ext(e.configFilePath) //obtain the extension of file
		if e.configExtension == "" {
			log.Fatalf("can't determine config file %s extension", e.configFilePath)
		}
	}

	viper.SetConfigFile(e.configFilePath)

	err = viper.ReadInConfig()
	if err != nil {
		log.Fatalf("error while reading config file: %v", err)
	}

	val, ok := viper.Get(key).(string)
	if !ok {
		log.Fatalf("Invalid type assertion")
	}

	return val
}

func (e *Environment) GetEnv(key, defaultValue string) *EnvVar {
	var envVar EnvVar
	envVar.envName = key

	if val := e.envConfigGet(key); val != "" {
		envVar.envValue = val
		return &envVar
	}

	if e.checkEnvVar {
		if val := os.Getenv(key); val != "" {
			envVar.envValue = val
			return &envVar
		} else {
			warning := fmt.Sprintf("env var %s not set", key)
			log.Println(highlightWarning(warning))
		}
	}

	if defaultValue != "" {
		envVar.envValue = defaultValue
		return &envVar
	}

	if e.nullable {
		warning := fmt.Sprintf("default value for env var %s not set but nullable values allowed. Returning empty string", key)
		log.Println(highlightWarning(warning))
	} else {
		log.Fatalf("env var %s not set. Default value is undefined", key)
	}
	envVar.envValue = ""
	return &envVar
}

func highlightWarning(warning string) string {
	return fmt.Sprintf("%s%s%s", yellow, warning, reset)
}
