using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace PipenGeneX
{
    public class TaskResult
    {
        public string TaskId { get; set; } = string.Empty;
        public string Command { get; set; } = string.Empty;
        public string StdOut { get; set; } = string.Empty;
        public ICollection<string> Errors { get; set; } = new List<string>();
        public bool Success => !Errors.Any();
    }
}
