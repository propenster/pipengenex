using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace PipenGeneX
{
    public class TaskCommand
    {
        public string Command { get; set; } = string.Empty;
        public string Id { get; set; } = Guid.NewGuid().ToString();
    }
}
