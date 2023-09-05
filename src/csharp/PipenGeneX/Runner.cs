using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace PipenGeneX
{
    public class Runner
    {
        public Workflow Workflow { get; set; } = new Workflow();
        public List<TaskResult> Results { get; set; } = new List<TaskResult>();
    }
}
