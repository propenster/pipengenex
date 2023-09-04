using CliWrap;
using CliWrap.Buffered;
using CommandLine;
using Newtonsoft.Json;
using PipenGeneX;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text.Json.Serialization;
using System.Text.RegularExpressions;

internal class Program
{

    public static void Main(string[] args)
    {
        Parser.Default.ParseArguments<Options>(args)
        .WithParsed(Run)
        .WithNotParsed(HandleErrors);
    }

    private static string[] GetCommands(string input)
    {
        if (string.IsNullOrWhiteSpace(input)) return new string[] { };
        int firstSpaceIndex = input.IndexOf(' ');
        var firstItem = string.Empty;
        var restAsOneItem = string.Empty;
        if (firstSpaceIndex != -1)
        {
            // Split the input into two parts
            firstItem = input.Substring(0, firstSpaceIndex);
            restAsOneItem = input.Substring(firstSpaceIndex + 1);

            Console.WriteLine("First Item: " + firstItem);
            Console.WriteLine("Rest as One Item: " + restAsOneItem);
        }
        return new string[] { firstItem, restAsOneItem };
    }

    static void Run(Options options)
    {
        Console.WriteLine("Workflow Definition File Path >>> {0}!", options.FILE);
        var runner = new Runner();

        //RunWorkFlow(options.FILE);

        //throw new CommandLine.Error(ErrorType.HelpRequestedError, options.FILE);
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
            PrintError(OptionsError.InvalidFile, "json");
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

        if (!(workflow.Sequence.Any() && workflow.Tasks.Any()))
        {
            PrintError(OptionsError.InvalidSequenceDefinitionError, string.Format("{0} / {1}", "Invalid Sequence of Tasks definition in workflow definition", options?.FILE));
            return;
        }

        foreach (var item in workflow?.Sequence)
        {
            if (!item.Trim().Contains(','))
            {
                var task = workflow?.Tasks.FirstOrDefault(x => x.Id == item.Trim());
                if (string.IsNullOrWhiteSpace(task?.Command))
                {
                    PrintError(OptionsError.InvalidTaskId, item.Trim());
                }
                var taskResults = Task.Run(async () => await RunTaskCommand(task?.Id, task?.Command,workflow?.WorkingDirectory, workflow?.Variables)).Result;
                runner.Results.Add(taskResults);
            }
            else
            {
                var tasks = workflow?.Tasks.Where(x => item.Trim().Split(',').Contains(x.Id)).ToArray();
                ParallelLoopResult res = Parallel.ForEach(tasks, async item => runner.Results.Add(await RunTaskCommand(item?.Id, item?.Command, workflow?.WorkingDirectory, workflow?.Variables)));
            }
        }

        //compile Report...

        return;
    }

    private static async Task<TaskResult> RunTaskCommand(string? id, string? commandToRun, string working_directory, Dictionary<string, string>? variables)
    {
        var taskResult = new TaskResult { Command = commandToRun, TaskId = id, };

        var globErrors = new List<string>();
        try
        {
            //get envs
            string pattern = @"\{\{([^}]+)\}\}";

            // Use Regex.Matches to find all matches in the input string
            if (variables.Any())
            {
                foreach (var item in variables)
                {
                    var match = Regex.Match(item.Value, pattern);
                    if (match.Success)
                    {
                        Console.WriteLine("Match: " + match.Groups[1].Value);
                        var envVariable = Environment.GetEnvironmentVariable(match.Groups[1].Value);
                        if (string.IsNullOrWhiteSpace(envVariable))
                        {
                            globErrors.Add(string.Format("Error. Could not retrieve Environment Variable configured in workflow definition {0}", match.Groups[0].Value));
                            //return new TaskResult
                            //{
                            //    Errors = new string[] {  }
                            //};
                            continue;
                        }
                        variables[item.Key] = envVariable;//  (match.Groups[0].Value, envVariable);
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
                    if(!variables.TryGetValue(variableName, out var variableValue))
                    {
                        //better way... Add to errors? and then return before you run command...
                        globErrors.Add(string.Format("Error. Could not retrieve Environment Variable configured in command text {0}", match.Groups[0].Value));
                    }
                    Console.WriteLine("Match: " + variableName);
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

            var cmd = Cli.Wrap(commandSplit.FirstOrDefault())
               .WithArguments(commandSplit.LastOrDefault())
               .WithWorkingDirectory(working_directory ?? Directory.GetCurrentDirectory());
            Console.WriteLine("Running command...");

            // Execute the command and capture the output
            var result = await cmd.ExecuteBufferedAsync();

            // Check the exit code to determine if the command succeeded or failed
            if (result.ExitCode == 0)
            {
                Console.WriteLine("command {0} executed successfully.", commandToRun);
                Console.WriteLine(result.StandardOutput);
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

    static string UnescapeBraces(string input)
    {
        // Replace escaped braces with their original form
        return input.Replace("\\{", "{").Replace("\\}", "}");
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
    static void HandleErrors(IEnumerable<Error> errors)
    {
        foreach (var error in errors)
        {
            Console.WriteLine($"Error: {error}");
        }
    }
}

public class Options
{
    [Option('f', "FILE", Required = true, HelpText = "PipegeneX Workflow definition JSON file")]
    public string FILE { get; set; }
}