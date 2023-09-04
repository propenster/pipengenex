using Newtonsoft.Json;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace PipenGeneX
{
    public class Workflow
    {
        [JsonProperty("name")]
        public string Name { get; set; } = string.Empty;
        [JsonProperty("description")]
        public string Description { get; set; } = string.Empty;
        [JsonProperty("working_directory")]
        public string WorkingDirectory { get; set; } = string.Empty;
        [JsonProperty("variables")]
        public HashSet<Variable> VariableList { get; set; } = new HashSet<Variable>();
        [JsonIgnore]
        public Dictionary<string, string> Variables { get; set; } = new Dictionary<string, string>();
        [JsonProperty("tasks")]
        public HashSet<TaskCommand> Tasks { get; set; } = new HashSet<TaskCommand>();
        [JsonProperty("run_sequence")]
        public HashSet<string> Sequence { get; set; } = new HashSet<string>();
    }
}
