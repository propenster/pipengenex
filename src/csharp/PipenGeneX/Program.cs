using Catel.IoC;
using CliWrap;
using CliWrap.Buffered;
using CommandLine;
using Newtonsoft.Json;
using Orc.SystemInfo;
using PipenGeneX;
using System.Text;
using System.Text.RegularExpressions;

internal class Program
{
    public static async Task Main(string[] args)
    {
        Console.WriteLine("Working... -->");
        var result = Parser.Default.ParseArguments<RunOptions, Options>(args);

        await result.MapResult(
            async (RunOptions opts) =>
            {
                await Run(new Options { FILE = opts.FilePath, Threads = opts.Threads });
            },
            errors => HandleErrors(errors));
        Console.WriteLine();
    }
    static Task HandleErrors(IEnumerable<Error> errors)
    {
        foreach (var error in errors)
        {
            switch (error.Tag)
            {
                case ErrorType.HelpVerbRequestedError:
                case ErrorType.HelpRequestedError:
                case ErrorType.VersionRequestedError:

                    continue;
                default:
                    Console.WriteLine($"Error: {error}");
                    break;

            }
        }
        return Task.CompletedTask;
    }
    private static string[] GetCommands(string input)
    {
        if (string.IsNullOrWhiteSpace(input)) return new string[] { };
        int firstSpaceIndex = input.IndexOf(' ');
        var firstItem = string.Empty;
        var restAsOneItem = string.Empty;
        if (firstSpaceIndex != -1)
        {
            firstItem = input.Substring(0, firstSpaceIndex);
            restAsOneItem = input.Substring(firstSpaceIndex + 1);
        }
        return new string[] { firstItem, restAsOneItem };
    }

    static async Task Run(Options options)
    {
        var runner = new Runner();

        var fileText = string.Empty;
        try
        {
            fileText = File.ReadAllText(options.FILE) ?? string.Empty;

        }
        catch (Exception ex)
        {

            PrintError(OptionsError.UnknownError, ex.Message);
        }

        if (string.IsNullOrWhiteSpace(fileText))
        {
            PrintError(OptionsError.InvalidFile, options?.FILE);
            return;
        }
        if (Path.GetExtension(options?.FILE).ToLowerInvariant() != ".json")
        {
            PrintError(OptionsError.InvalidFile, ".json");
            return;
        }

        var workflow = JsonConvert.DeserializeObject<Workflow>(fileText);
        if (workflow == null)
        {
            PrintError(OptionsError.InvalidFile, options?.FILE);
            return;
        }
        workflow.Variables = workflow.VariableList.ToDictionary(x => x.Key, x => x.Value); //convert it now to dictionary...
        runner.Workflow = workflow;
        workflow.WorkingDirectory = !Directory.Exists(workflow?.WorkingDirectory) ? Directory.GetCurrentDirectory() : workflow.WorkingDirectory;

        if (!(workflow.Sequence.Any() && workflow.Tasks.Any()))
        {
            PrintError(OptionsError.InvalidSequenceDefinitionError, string.Format("{0} / {1}", "Invalid Sequence of Tasks definition in workflow definition", options?.FILE));
            return;
        }

        await RunWorkflow(runner, workflow);

        //compile Report...
        Console.WriteLine("Tasks Run Completed...\\/\\/");
        Console.WriteLine("Compiling Report ...");

        var systemInfoService = ServiceLocator.Default.ResolveType<ISystemInfoService>();
        var sysInfo = systemInfoService.GetSystemInfo().Where(x => !string.IsNullOrWhiteSpace(x.Name) && !string.IsNullOrWhiteSpace(x.Value)).ToDictionary(x => x.Name, x => x.Value);
        GenerateReport(runner, workflow, sysInfo);
        return;
    }

    private static async Task RunWorkflow(Runner runner, Workflow? workflow)
    {
        foreach (var item in workflow?.Sequence)
        {
            if (!item.Trim().Contains(','))
            {
                var task = workflow?.Tasks.FirstOrDefault(x => x.Id == item.Trim());
                if (string.IsNullOrWhiteSpace(task?.Command))
                {
                    PrintError(OptionsError.InvalidTaskId, item.Trim());
                }
                var taskResults = await RunTaskCommand(task?.Id, task?.Command, workflow?.WorkingDirectory, workflow?.Variables);
                runner.Results.Add(taskResults);
            }
            else
            {
                var tasks = workflow?.Tasks.Where(x => item.Trim().Split(',').Contains(x.Id)).ToArray();
                ParallelLoopResult res = Parallel.ForEach(tasks, async item => runner.Results.Add(await RunTaskCommand(item?.Id, item?.Command, workflow?.WorkingDirectory, workflow?.Variables)));
            }
        }
    }

    private static void GenerateReport(Runner runner, Workflow? workflow, Dictionary<string, string> sysInfo)
    {
        var outputReportPath = Path.Combine(workflow?.WorkingDirectory, "report.csv");

        try
        {
            File.Create(outputReportPath).Close();
            File.AppendAllText(outputReportPath, string.Format("System Information:{0}{1}{2}{3}{4}{5}{6}", Environment.NewLine, "==============================", Environment.NewLine, string.Join(",", sysInfo.Keys.ToArray()), Environment.NewLine, string.Join(",", sysInfo.Values.ToArray()), Environment.NewLine));

            //write csv of taskResults...
            File.AppendAllText(outputReportPath, string.Format("TASK_ID,COMMAND,OUTPUT,ERROR,SUCCESSFUL{0}", Environment.NewLine));
            if (runner.Results != null && runner.Results.Any())
            {
                var sb = new StringBuilder();
                foreach (var item in runner?.Results)
                {
                    sb.AppendLine(string.Format("{0},{1},{2},{3},{4}", item?.TaskId, item?.Command, string.Empty, string.Join(";", item?.Errors) ?? string.Empty, item.Success == true ? 1 : 0));
                }

                File.AppendAllText(outputReportPath, sb.ToString());
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine("Error: {0}", ex.Message);
        }
    }

    private static async Task<TaskResult> RunTaskCommand(string? id, string? commandToRun, string workingDirectory, Dictionary<string, string>? variables)
    {
        Console.WriteLine("Runing... \\/\\/");
        Console.WriteLine();
        var taskResult = new TaskResult { Command = commandToRun, TaskId = id, };

        var globErrors = new List<string>();
        try
        {
            //get envs
            string pattern = @"\{\{([^}]+)\}\}";

            // Use Regex.Matches to find all matches in the input string
            if (variables != null && variables.Any())
            {
                foreach (var item in variables)
                {
                    var match = Regex.Match(item.Value, pattern);
                    if (match.Success)
                    {
                        var envVariable = Environment.GetEnvironmentVariable(match.Groups[1].Value);
                        if (string.IsNullOrWhiteSpace(envVariable))
                        {
                            globErrors.Add(string.Format("Error. Could not retrieve Environment Variable configured in workflow definition {0}", match.Groups[0].Value));
                            continue;
                        }
                        variables[item.Key] = envVariable;
                    }

                }
            }

            // Iterate through the matches and print them
            string commandVarPattern = @"\$\{([^}]+)\}";

            var areBracesEscaped = commandToRun.Contains("{{");

            //if (areBracesEscaped) commandToRun = UnescapeBraces(commandToRun);

            // Use Regex.Matches to find all matches in the input string
            MatchCollection matches = Regex.Matches(commandToRun, commandVarPattern);

            // Iterate through the matches and print them
            if (matches.Any())
            {
                foreach (Match match in matches)
                {
                    string variableName = match.Groups[1].Value;
                    if (!variables.TryGetValue(variableName, out var variableValue))
                    {
                        //better way... Add to errors? and then return before you run command...
                        globErrors.Add(string.Format("Error. Could not retrieve Environment Variable configured in command text {0}", match.Groups[0].Value));
                    }
                    //Console.WriteLine("Match: " + variableName);
                    commandToRun = commandToRun.Replace(match.Value, variableValue);

                }
            }
            //also check for variables...

            //every error before this line will affect our command... so let's short-circuit
            if (globErrors.Any())
            {
                taskResult.Errors = globErrors;
                return taskResult;
            }

            var commandSplit = GetCommands(commandToRun);
            if (!commandSplit.Any())
            {
                globErrors.Add("Invalid command");
                return taskResult;
            }

            //if (!Directory.Exists(working_directory)) working_directory = Directory.GetCurrentDirectory();

            var cmd = Cli.Wrap(commandSplit.FirstOrDefault())
               .WithArguments(commandSplit.LastOrDefault())
               .WithWorkingDirectory(workingDirectory);
            Console.WriteLine("Running Task command... {0}", commandToRun);

            // Execute the command and capture the output
            var result = await cmd.ExecuteBufferedAsync();

            // Check the exit code to determine if the command succeeded or failed
            if (result.ExitCode == 0)
            {
                Console.WriteLine("command {0} executed successfully.", commandToRun);
                Console.WriteLine(result.StandardOutput);
                taskResult.StdOut = result.StandardOutput;
            }
            else
            {
                Console.WriteLine("command {0} failed.", commandToRun);
                Console.WriteLine(result.StandardError);
                taskResult.Errors.Add(result.StandardError);
            }

            return taskResult;
        }
        catch (Exception ex)
        {
            taskResult.Errors.Add(ex.Message);
            return taskResult;
        }
    }
    public static void PrintError(OptionsError error, string? message)
    {
        Console.ForegroundColor = ConsoleColor.Red;
        switch (error)
        {
            case OptionsError.InvalidFile:
                Console.WriteLine("The file specified as 'FILE': {0} is invalid. It may be empty or its content is not properly encoded", message);
                break;

            case OptionsError.InvalidFileFormat:
                Console.WriteLine("The file specified must be in a specific format. Accepted workflow definition file format: {0}", message);
                break;

            case OptionsError.UnparseableCommand:
                Console.WriteLine("The command {0} is invalid or unparseable.", message);
                break;

            case OptionsError.FileDoesNotExist:
                Console.WriteLine("The file does not exist.");
                break;
            case OptionsError.InvalidTaskId:
                Console.WriteLine("The taskID {0} does not exist.", message);
                break;

            case OptionsError.InvalidSequenceDefinitionError:
                Console.WriteLine("The sequence definition in the workflow definition JSON is invalid.");
                break;

            case OptionsError.CommandFailed:
                Console.WriteLine("The command is invalid or unparseable.");
                break;

            case OptionsError.UnknownError:
                Console.WriteLine("An unknown error has occurred. Please find details here: {0}", message);
                break;

            default:
                Console.WriteLine("Unknown error");
                break;
        }
        Console.ResetColor();
    }

}

