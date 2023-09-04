using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace PipenGeneX
{
    public enum OptionsError
    {
        InvalidFile,

        InvalidFileFormat,

        UnparseableCommand,

        FileDoesNotExist,

        InvalidSequenceDefinitionError,

        CommandFailed,
        UnknownError,
        InvalidTaskId,
    }

    public class ErrorAttribute : Attribute
    {
        public string Message { get; }

        public ErrorAttribute(string message)
        {
            Message = message;
            Console.ForegroundColor = ConsoleColor.Red;
            Console.Error.WriteLine(Message);
            Console.ResetColor();
        }
    }

}
