using CommandLine.Text;
using CommandLine;
using Newtonsoft.Json;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace PipenGeneX
{
    [Verb("run", HelpText = "Run a command with a file path")]
    public class RunOptions
    {
        [Value(0, Required = true, HelpText = "File path")]
        public string FilePath { get; set; } = string.Empty;

        [Option(shortName: 't', "threads", HelpText = "Maximum number of threads")]
        public int Threads { get; set; }
    }


    //[Verb("run", HelpText = "Run a PipegeneX Workflow definition JSON filePath")]
    public class Options
    {
        //[Option('f', "FILE", Required = true, HelpText = "PipegeneX Workflow definition JSON file")]
        //[Value(0, Required = true, HelpText = "PipegeneX Workflow definition JSON file")]
        //public string FILE { get; set; } = string.Empty;
        //[Value(0, MetaName = "run", Required = true, HelpText = "Run a PipegeneX Workflow definition JSON file")]
        //public string run { get; set; }
        
        [Value(0, MetaName = "FILE", Required = true, HelpText = "PipegeneX Workflow definition JSON file")]
        public string FILE { get; set; } = string.Empty;

        //[Value(0, MetaName = "FILEPath", Required = false, HelpText = "File path")]
        //public string FilePath { get; set; }

        //[Value(0, MetaName = "FILEPath", Required = false, HelpText = "File path")]
        [Option(shortName:'t', "threads", HelpText = "Maximum number of threads")]
        public int Threads { get; set; }

    }
}
