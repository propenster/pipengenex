use clap::{builder::NonEmptyStringValueParser, Arg, ArgAction, ArgMatches, Command};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    ffi::{OsStr, OsString},
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    process::Stdio,
    thread,
    time::Duration,
};
use thiserror::Error;

use anyhow::Result;
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
    fn new(task_id: String, command: String, errors: &[String]) -> TaskResult {
        // let task_result = TaskResult::default();
        let success = if errors.len() > 0 { false } else { true };

        TaskResult {
            task_id: Some(task_id),
            command: Some(command.to_owned()),
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
    fn run_flow(&mut self) -> Result<(), OptionsError> {
        //let (tx, rx) = mpsc::channel();

        let workflow = self.options.workflow.clone().unwrap();
        let mut ntasks = workflow.tasks.clone();
        if &workflow.sequence.len() > &0 {
            for id_str in &workflow.sequence {
                //if item.is_string() {
                // If it's a string, check for commas and split if necessary
                if id_str.contains(',') {
                    // Split by commas and let each taskCommand run on it's own thread...

                    run_parallel(id_str, workflow).unwrap();
        
                    //end thread spawn
                } else {
                    let id = id_str.to_owned();
                    // Add as a single TaskCommand
                    let matching_task_commands: Vec<TaskCommand> = ntasks
                        // Clone the Vec<TaskCommand> to avoid moving it
                        .into_iter()
                        .filter(|task| id_str.contains(&task.id))
                        .collect();
                    if let Some(first) = matching_task_commands.first() {
                        let i = first.clone().to_owned();
                        match run_command(&i.id, &i.command_to_run) {
                            Ok(e) => self.results.push(e),
                            Err(e) => {
                                return Err(OptionsError::CommandFailed);
                            }
                        }
                    }
                }
                // } else {
                //     return Err("Invalid element in 'run_sequence'".into());
                // }
            }
        } else {
            return Err(OptionsError::InvalidSequenceDefinitionError(
                "Invalid element in 'run_sequence'".into(),
            ));
        }

        //loop through the sequence Vec...
        //start pushing to std::process::CMD
        // if one iter is comma-delimited, split them and pool them to run in parallel...
        // push to results: Vec<TaskResult> as you iterate through each TASK...

        Ok(())
    }
    

    // fn run_workflow(&mut self) -> Result<(), OptionsError> {
    //     let (tx, rx) = mpsc::channel();

    //     let workflow = self.options.workflow.clone().unwrap();
    //     let mut ntasks = workflow.tasks.clone();
    //     if workflow.sequence.len() > 0 {
    //         for id_str in workflow.sequence {
    //             if id_str.contains(',') {
    //                 let matching_task_commands: Vec<TaskCommand> = workflow
    //                     .tasks
    //                     .into_iter()
    //                     .filter(|task| id_str.split(',').any(|part| part.trim() == task.id))
    //                     .collect();

    //                 let handle = thread::spawn(move || {
    //                     // value to be sent
    //                     for task in matching_task_commands {
    //                         // match run_command(task.id, task.command_to_run) {
    //                         //     Ok(e) => Ok(tx.send(e).unwrap()),
    //                         //     Err(e) => {
    //                         //         return Err(OptionsError::CommandFailed);
    //                         //     }
    //                         // }
    //                         let i = run_command(task.id, task.command_to_run).unwrap();
    //                         tx.send(i);
    //                         thread::sleep(Duration::from_millis(500));
    //                     }
    //                 });
    //                 handle.join().unwrap();

    //                 for i in rx {
    //                     self.results.push(i);
    //                     thread::sleep(Duration::from_millis(500));
    //                 }

    //                 // catch the value with the recv()
    //                 // function and store it in 'b'
    //                 //let b:i32 = rx.recv().unwrap();
    //             } else {
    //                 let matching_task_commands: Vec<TaskCommand> = ntasks
    //                     .into_iter()
    //                     .filter(|task| id_str.contains(&task.id))
    //                     .collect();
    //                 if let Some(first) = matching_task_commands.first() {
    //                     let i = first.clone().to_owned();
    //                     match run_command(id_str, i.command_to_run) {
    //                         Ok(e) => self.results.push(e),
    //                         Err(e) => {
    //                             return Err(OptionsError::CommandFailed);
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     } else {
    //         return Err(OptionsError::InvalidSequenceDefinitionError(
    //             "Invalid element in 'run_sequence'".into(),
    //         ));
    //     }

    //     Ok(())
    // }

    fn compile_report(&mut self) -> Result<()> {
        Ok(())
    }
}


fn run_parallel(id_str: String, workflow: Workflow) -> Result<()>{

    let mut res: Vec<TaskResult> = vec![];
    let (tx, rx) = mpsc::channel();

                    let handle = thread::spawn(move || {
                        // value to be sent
                        let matching_task_commands: Vec<TaskCommand> = workflow
                            .tasks
                            .into_iter()
                            .filter(|task| id_str.split(',').any(|part| part.trim() == task.id))
                            .collect();

                        //loop
                        for task in matching_task_commands {
                            // catch the value with the recv()
                            // function and store it in 'b'
                            //let b:i32 = rx.recv().unwrap();

                            let t = run_command(&task.id, &task.command_to_run).unwrap();
                            tx.send(t).unwrap();
                            thread::sleep(Duration::from_millis(500));


                            //self.results.push(rx.recv().unwrap());
                        }

                        
                    });

                    handle.join().unwrap();
                    for i in rx {
                        res.push(i);
                        thread::sleep(Duration::from_millis(500));
                    }


                    Ok(())

                    
}

fn run_command(task_id: &str, command: &str) -> Result<TaskResult, Box<dyn std::error::Error>> {
    let cmd = std::process::Command::new(command)
        .stderr(Stdio::piped())
        .output()?;

    if cmd.status.success() {
        println!("Command executed successfully");
        Ok(TaskResult::new(String::from(task_id), String::from(command), &[]))
    } else {
        eprintln!("Failed to execute command");
        let mut errors = Vec::new();
        for e in cmd.stderr {
            errors.push(e.to_string());
        }
        //Err(OptionsError::InvalidCommandError(command))
        Ok(TaskResult::new(String::from(task_id), String::from(""), &errors))
    }
}

#[derive(Debug, thiserror::Error)]
enum OptionsError {
    #[error("The file '{0}' specified as 'FILE' is invalid. ")]
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

    let workflow: Workflow = serde_json::from_str(&file_text)
        .map_err(|_| OptionsError::InvalidFile(file_path.to_string_lossy().to_string()))?;

    Ok(workflow)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TaskCommand {
    command_to_run: String,
    id: String,
}
impl TaskCommand {
    fn new(command: String, id: String) -> Self {
        TaskCommand {
            command_to_run: command,
            id: id,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Options {
    workflow_file_path: PathBuf,
    workflow: Option<Workflow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Variable {
    key: Option<String>,
    value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Workflow {
    name: Option<String>,
    description: Option<String>,
    variables: Vec<Variable>,
    tasks: Vec<TaskCommand>,
    sequence: Vec<String>,
}
impl Default for Workflow {
    fn default() -> Self {
        Workflow {
            name: Some("Workflow1".into()),
            description: None,
            variables: Vec::new(),
            tasks: Vec::new(),
            sequence: Vec::new(),
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

        // let mut reader = match matches.get_one::<String>("FILE") {
        //     Some(filename) => Input::File(File::open(filename)?),
        //     None => Input::Stdin(stdin.lock()),
        // };

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

        let workflow_from_pathbuf = build_workflow_from_file(options.workflow_file_path)?;

        Ok(options)
    }
}

fn run() -> Result<()> {
    let cli_arguments = get_cli_args(env::args_os());
    let options = Options::from_cli_arguments(&cli_arguments)?;

    let mut runner = Runner::new(options);
    runner.run_flow()?;
    runner.compile_report()?;

    Ok(())
}

fn main() {
    println!("Hello, world!");
    match run() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{} {:#}", "Error:", e);
            std::process::exit(1);
        }
    }
}
