use clap::{builder::NonEmptyStringValueParser, Arg, ArgAction, ArgMatches, Command};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    ffi::{OsStr, OsString},
    fmt::Display,
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    os::windows,
    path::{Path, PathBuf},
    process::Stdio,
    thread,
    time::Duration,
};
use thiserror::Error;

use anyhow::Result;
use regex::Regex;
use serde_json::{self, Value};
use std::sync::mpsc;
// use std::thread;

// Get CLI Arguments
// Get Options From CLI Arguments
//

//get workflow_definition_json
// get working directory...
// get environment variables
// substitute environment variables...
// get taskList
// get Sequence

// parse Sequence...

#[derive(Debug, Clone)]
struct TaskResult {
    task_id: Option<String>,
    command: Option<String>,
    errors: Vec<String>,
    success: bool,
}
impl Default for TaskResult {
    fn default() -> Self {
        TaskResult {
            task_id: None,
            command: None,
            errors: Vec::new(),
            success: false,
        }
    }
}
impl TaskResult {
    fn new(
        task_id: impl Into<String>,
        command: impl Into<String>,
        errors: &[String],
    ) -> TaskResult {
        // let task_result = TaskResult::default();
        let success = if errors.len() > 0 { false } else { true };

        TaskResult {
            task_id: Some(task_id.into()),
            command: Some(command.into()),
            errors: Vec::new(),
            success,
        }
    }
}

#[derive(Debug, Clone)]
struct Runner {
    options: Options,

    results: Vec<TaskResult>,
}
impl Runner {
    fn new(options: Options) -> Self {
        Runner {
            options: options,
            results: vec![],
        }
    }

    fn run_new_flow(&mut self) -> Result<(), OptionsError> {
        let mut workflow = self.options.workflow.clone().unwrap();
        let ntasks = workflow.tasks.clone(); // Clone ntasks here
        let wd = workflow.working_directory.as_ref().unwrap().clone();

        //replace environment variables defined in the Workflow definition JSON file...
        let vars = workflow.environment_variables.clone();
        let (transformed_vars, errors) = match replace_variables_with_actual_var_values(vars) {
            Ok((a, b)) => {
                println!("Successfully parsed variables...");
                (a, b)
            }
            Err(e) => {
                println!("Error while fetching environment variables >>> {}", e);
                let a = HashMap::new();
                let e = vec![e.to_string()];
                (a, e)
            }
        };
        workflow.environment_variables = transformed_vars; //after environment variables in definition are fetched...
        if !errors.is_empty() {
            eprintln!("Error fetching environment variables from host");
            return Err(OptionsError::InAccessibleEnvironmentVariableError);
        }

        //now we have substituted the environment defined with actual values from os...
        //let's now subsctitute environment variables probably used in our task commandString?

        if !workflow.run_sequence.is_empty() {
            for id_str in &workflow.run_sequence {
                if id_str.contains(',') {
                    // Split by commas and let each taskCommand run on its own thread...
                    run_parallel(id_str, &workflow).unwrap();
                    // end thread spawn
                } else {
                    let id = id_str.to_owned();
                    // Add as a single TaskCommand
                    let matching_task_commands: Vec<TaskCommand> = ntasks
                        .iter()
                        .filter(|task| id_str.contains(&task.id))
                        .cloned()
                        .collect();
                    if let Some(first) = matching_task_commands.first() {
                        let i = first.clone().to_owned();
                        match run_command(
                            &i.id,
                            &i.command_to_run,
                            &wd,
                            &workflow.environment_variables,
                        ) {
                            Ok(e) => self.results.push(e),
                            Err(e) => {
                                eprintln!("Error while running command : >>> {}", e);
                                return Err(OptionsError::CommandFailed);
                            }
                        }
                    }
                }
            }
        } else {
            return Err(OptionsError::InvalidSequenceDefinitionError(
                "Invalid element in 'run_sequence'".into(),
            ));
        }

        Ok(())
    }

    fn compile_report(&mut self) -> Result<(), OptionsError> {
        let work_dir = &self.options.workflow.unwrap().working_directory.unwrap();
        let output_report_path = Path::new(&work_dir)
            .join("report.csv")
            .to_string_lossy()
            .to_string();

        if let Err(err) = File::create(&output_report_path) {
            eprintln!("Error creating file: {}", err);
            return Err(OptionsError::ReportError(format!("{} FilePath: {}", err, output_report_path)))
        }

        let mut file = match File::open(&output_report_path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Error opening file: {}", err);
                return Err(OptionsError::ReportError(format!("{} FilePath: {}", err, output_report_path)))
            }
        };

        // if let Err(err) = writeln!(
        //     file,
        //     "System Information:\n==============================\n{}\n{}\n{}\n",
        //     sys_info.keys().collect::<Vec<_>>().join(","),
        //     sys_info.values().collect::<Vec<_>>().join(","),
        //     ""
        // ) {
        //     eprintln!("Error writing to file: {}", err);
        //     return Err(OptionsError::ReportError(format!("{} FilePath: {}", err, output_report_path)))
        // }

        if let Err(err) = writeln!(file, "TASK_ID,COMMAND,OUTPUT,ERROR,SUCCESSFUL\n") {
            eprintln!("Error writing to file: {}", err);
            return Err(OptionsError::ReportError(format!("{} FilePath: {}", err, output_report_path)))
        }

        if let Some(results) = self.results {
            let mut sb = String::new();

            for item in results {
                sb.push_str(&format!(
                    "{},{},{},{},{}\n",
                    item.task_id.unwrap_or_default(),
                    item.command.as_deref().unwrap_or_default(),
                    "",
                    item.errors
                        .as_ref()
                        .map(|errors| errors.join(";"))
                        .unwrap_or_default(),
                    if item.success.unwrap_or_default() {
                        "1"
                    } else {
                        "0"
                    }
                ));
            }

            if let Err(err) = file.write_all(sb.as_bytes()) {
                eprintln!("Error writing to file: {}", err);
                return Err(OptionsError::ReportError(format!("{} FilePath: {}", err, output_report_path)))
            }
        }

        Ok(())
    }
}

fn replace_variables_main_task_command_string(
    variables: HashMap<String, String>,
    command_to_run: &str,
) -> Result<(String, Vec<String>), Box<dyn std::error::Error>> {
    let command_var_pattern = r"\$\{([^}]+)\}";
    let re = Regex::new(command_var_pattern).expect("Regex compilation failed");

    // Sample variables
    // let mut variables: HashMap<String, String> = HashMap::new();
    // variables.insert("variable1".to_string(), "value1".to_string());
    // variables.insert("variable2".to_string(), "value2".to_string());

    let mut glob_errors: Vec<String> = Vec::new();

    let replaced_command = re.replace_all(command_to_run, |caps: &regex::Captures| {
        let variable_name = caps.get(1).unwrap().as_str();
        if let Some(variable_value) = variables.get(variable_name) {
            variable_value.to_string()
        } else {
            glob_errors.push(format!(
                "Error. Could not retrieve Environment Variable configured in command text {}",
                caps.get(0).unwrap().as_str()
            ));
            String::from(caps.get(0).unwrap().as_str())
        }
    });

    let res = String::from(replaced_command);

    println!("Replaced Command: {}", res);

    Ok((res, glob_errors))
}

fn replace_variables_with_actual_var_values(
    mut variables: HashMap<String, String>,
) -> Result<(HashMap<String, String>, Vec<String>), Box<dyn std::error::Error>> {
    let pattern = r"\{\{([^}]+)\}\}";
    let re = Regex::new(pattern).expect("Regex compilation failed");

    let mut glob_errors: Vec<String> = Vec::new();

    for (key, value) in &mut variables {
        if let Some(captures) = re.captures(value) {
            let env_variable_name = &captures[1];
            if let Ok(env_variable) = env::var(env_variable_name) {
                if !env_variable.is_empty() {
                    *value = env_variable;
                } else {
                    glob_errors.push(format!(
                        "Error. Could not retrieve Environment Variable configured in workflow definition {}",
                        &captures[0]
                    ));
                }
            }
        }
    }

    Ok((variables, glob_errors))
}

fn run_parallel(id_str: &str, workflow: &Workflow) -> Result<()> {
    let mut res: Vec<TaskResult> = vec![];
    let wd = workflow.working_directory.as_ref().unwrap().clone();
    let (tx, rx) = mpsc::channel();

    let id_str_clone = id_str.to_string();
    let workflow_clone = workflow.clone();
    let handle = thread::spawn(move || {
        let matching_task_commands: Vec<TaskCommand> = workflow_clone
            .tasks
            .into_iter()
            .filter(|task| id_str_clone.split(',').any(|part| part.trim() == task.id))
            .collect();

        //loop
        for task in matching_task_commands {
            let t = run_command(
                &task.id,
                &task.command_to_run,
                &wd,
                &workflow_clone.environment_variables,
            )
            .unwrap();
            tx.send(t).unwrap();
            thread::sleep(Duration::from_millis(500));
        }
    });

    handle.join().unwrap();
    for i in rx {
        res.push(i);
        thread::sleep(Duration::from_millis(500));
    }

    Ok(())
}

fn get_commands(input: &str) -> [&str; 2] {
    if input.is_empty() {
        return ["", ""];
    }

    if let Some(first_space_index) = input.find(' ') {
        let (first_item, rest_as_one_item) = input.split_at(first_space_index);
        [first_item.trim(), rest_as_one_item.trim_start()]
    } else {
        [input.trim(), ""]
    }
}

fn run_command(
    task_id: &str,
    command: &str,
    working_directory: &str,
    variables: &HashMap<String, String>,
) -> Result<TaskResult, Box<dyn std::error::Error>> {
    let mut command_string: &str = command;
    let mut cmd_val: String = String::new();
    if !variables.is_empty() {
        let vars = variables.clone();
        let (cmd, error) = match replace_variables_main_task_command_string(vars, command) {
            Ok((a, b)) => {
                println!("Successfully swapped environment variables in command string");

                (a, b)
            }
            Err(e) => {
                eprintln!("Could not swap out environment variables in command string");
                let a = String::new();
                let errors = vec![e.to_string()];
                (a, errors)
            }
        };

        if error.is_empty() && !cmd.is_empty() {
            //successful...
            cmd_val = String::from(cmd);
            command_string = &cmd_val;
        } else {
            return Ok(TaskResult::new(
                String::from(task_id),
                String::from(command),
                &error,
            ));
        }
    }

    //println!("CMD Val >>> {}", cmd_val);
    //command_string = cmd_val;

    let [first_item, rest_as_one_item] = get_commands(command_string);

    // For Windows
    #[cfg(windows)]
    let mut cmd = {
        let mut command = std::process::Command::new("cmd");
        command.arg("/c").arg(first_item);
        command
    };

    // For Unix et al.
    #[cfg(not(windows))]
    let mut cmd = {
        let mut command = std::process::Command::new(first_item);
        //command.arg(first_item);
        command
    };

    for arg in rest_as_one_item.split_whitespace() {
        cmd.arg(arg);
    }

    cmd.current_dir(working_directory)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn()?;

    match child.wait_with_output() {
        Ok(s) => {
            println!("Command Status: {}", s.status);

            if s.status.success() {
                println!("Command executed successfully");
                return Ok(TaskResult::new(task_id, command, &[]));
            } else {
                eprintln!("Failed to execute command: '{}'", command);
                let mut errors = Vec::new();
                errors.push(format!("{}", String::from_utf8_lossy(&s.stderr)));

                return Ok(TaskResult::new(
                    String::from(task_id),
                    String::from(command),
                    &errors,
                ));
            }
        }
        Err(e) => {
            println!(
                "Error occurred while running command: '{}' Error Info: {}",
                command, e
            );
            return Ok(TaskResult::new(
                String::from(task_id),
                String::from(command),
                &vec![e.to_string()],
            ));
        }
    };
}

#[derive(Debug, thiserror::Error)]
enum OptionsError {
    #[error("The file '{0}' specified as 'FILE' is invalid. Failed to deserialize json")]
    InvalidFile(String),
    #[error("The file '{0}' specified must be in '{0}' format")]
    InvalidFileFormat(String, String),
    #[error("The command '{0}' is invalid or unparseable")]
    UnparseableCommand(String),
    #[error("The file '{0}' does not exist")]
    FileDoesNotExist(String),
    #[error("The sequence definition in the workflow definition JSON is invalid")]
    InvalidSequenceDefinitionError(String),
    #[error("The command is invalid or unparseable")]
    CommandFailed,
    #[error("Could not fetch or parse environment variables from run host")]
    InAccessibleEnvironmentVariableError,
    #[error("Error generating run report: '{0}'")]
    ReportError(String),
}

// impl Display for OptionsError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match *self{
//             Self::InvalidFile(String) => write!(f, "Invalid File '{}'. It may that the workflow definition / JSON file is incorrectly filled or formatted according to rules and standards", self.0),
//             Self::UnparseableCommand(String) => write!(f, "A task command is not shell parseable. / Please check the command '{}' and run the workflow pipeline again. ", self.0),
//             Self::InvalidFileFormat => write!(f, "The file '{}' specified must be in '{}' format", self.0, self.1),
//             Self::FileDoesNotExist => write!(f, "The file '{}' does not exist. Please fix the file paths and try run workflow again.", self.0),

//         }
//     }
// }

fn get_command() -> Command {
    Command::new("pipegenex")
        //.version(crate_version!())
        .next_line_help(true)
        .about("A command-line tool that is part of a new framework for generating genomics and bioinformatics analysis pipelines.")
        .help_expected(true)
        .max_term_width(80)
        .arg(
            Arg::new("FILE")
                .help("The workflow definition JSON file")
                .required(true)
                .action(ArgAction::Append)
        )
}

pub fn get_cli_args<'a, T, R>(args: T) -> ArgMatches
where
    T: IntoIterator<Item = R>,
    R: Into<OsString> + Clone + 'a,
{
    let command = get_command();
    command.get_matches_from(args)
}
fn build_workflow_from_file<A: AsRef<Path>>(file: A) -> Result<Workflow, OptionsError> {
    let file_path = file.as_ref();
    let file_text = fs::read_to_string(file_path)
        .map_err(|_| OptionsError::InvalidFile(file_path.to_string_lossy().to_string()))?;
    //println!("{}", file_text);

    // let workflow: Workflow = serde_json::from_str(&file_text)
    //     .map_err(|e| OptionsError::InvalidFile(file_path.to_string_lossy().to_string()))?;

    let mut workflow = match serde_json::from_str(&file_text) {
        Ok(b) => {
            // Deserialize the JSON into a struct
            let data: Value = serde_json::from_str(&file_text).expect("JSON parsing failed");

            // Extract the "variables" field from the JSON data
            let variables_json = data["variables"]
                .as_array()
                .expect("Invalid JSON structure");
            let mut variables_map: HashMap<String, String> = HashMap::new();

            for variable_json in variables_json {
                let variable: Variable = serde_json::from_value(variable_json.clone())
                    .expect("JSON deserialization failed");
                variables_map.insert(variable.key, variable.value);
            }
            let mut x: Workflow = b;

            x.environment_variables = variables_map;
            x
        }
        Err(e) => {
            panic!("Error occurred while deserializing workflow file: {}", e);
        }
    };

    Ok(workflow)
}
#[derive(Debug, Clone, Deserialize)]
struct TaskCommand {
    id: String,
    #[serde(rename = "command")]
    command_to_run: String,
    description: String,
}
impl TaskCommand {
    fn new(id: &'static str, command: &'static str, description: &'static str) -> Self {
        TaskCommand {
            command_to_run: command.to_string(),
            id: id.to_string(),
            description: description.to_string(),
        }
    }
}
#[derive(Debug, Clone, Deserialize)]
struct Options {
    workflow_file_path: PathBuf,
    workflow: Option<Workflow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Variable {
    key: String,
    value: String,
}

#[derive(Debug, Clone, Deserialize)]
struct Workflow {
    name: Option<String>,
    description: Option<String>,
    working_directory: Option<String>,
    variables: Vec<Variable>,
    tasks: Vec<TaskCommand>,
    #[serde(rename = "run_sequence")]
    run_sequence: Vec<String>,
    #[serde(skip)]
    environment_variables: HashMap<String, String>,
}
impl Default for Workflow {
    fn default() -> Self {
        Workflow {
            name: Some("Workflow1".into()),
            description: None,
            working_directory: None,
            variables: Vec::new(),
            tasks: Vec::new(),
            run_sequence: Vec::new(),
            environment_variables: HashMap::new(),
        }
    }
}

impl Default for Options {
    fn default() -> Options {
        Options {
            workflow_file_path: PathBuf::new(),
            workflow: None,
        }
    }
}

impl Options {
    fn from_cli_arguments(matches: &ArgMatches) -> Result<Self, OptionsError> {
        let mut options = Self::default();
        options.workflow_file_path = matches
            .get_one::<String>("FILE")
            .map(|path_str| {
                let path = PathBuf::from(path_str);
                if !path.exists() {
                    return Err(OptionsError::FileDoesNotExist(path_str.to_string()));
                }
                if path.extension().and_then(OsStr::to_str) != Some("json") {
                    return Err(OptionsError::InvalidFileFormat(
                        path_str.to_string(),
                        "json".into(),
                    ));
                }
                Ok(path)
            })
            .unwrap_or(Ok(Default::default()))?;

        let workflow_from_pathbuf = build_workflow_from_file(&options.workflow_file_path)?;
        options.workflow = Some(workflow_from_pathbuf);

        Ok(options)
    }
}

fn run() -> Result<()> {
    let cli_arguments = get_cli_args(env::args_os());
    let options = Options::from_cli_arguments(&cli_arguments)?;

    let mut runner = Runner::new(options);
    runner.run_new_flow()?;
    runner.compile_report()?;

    Ok(())
}

fn main() {
    let argss = env::args_os();
    println!("Hello, world!");
    match run() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{} {:#}", "Error:", e);

            std::process::exit(1);
        }
    }
}
