# pipengenex

## Overview

**pipengenex** is a powerful framework designed for creating and executing Bioinformatic pipeline workflows. It includes a Command-Line Utility (CLI) tool for effortlessly running pipeline workflows and tasks defined through JSON workflow definitions. Some notable features of pipengenex include task parallelization and flexible workflow configuration.

## Table of Contents

- [Getting Started](#getting-started)
- [Defining Workflow](#defining-workflow)
- [Running Workflows](#running-workflows)
- [Report and Error Handling](#report-and-error-handling)
- [Examples](#examples)
- [Contributing](#contributing)
- [License](#license)

## Getting Started

To get started with **pipengenex**, you can install it using platform-specific binaries available in the [Releases Page](https://github.com/your-username/your-repo/releases).

### Using Platform-Specific Binaries

1. Go to the [Releases Page](https://github.com/propenster/pipengenex/releases).

2. Download the appropriate binary for your operating system (e.g., `pipengenex.exe` for Windows, `pipengenex-linux` for Linux, or `pipengenex-macos` for macOS).

3. Place the downloaded binary in a directory included in your system's `PATH` environment variable.

4. Open your terminal or command prompt and verify the installation by running:

   ```bash
   pipengenex --version


## Running Your First Workflow

Now that you have **pipengenex** installed, it's time to create and execute your first Bioinformatic pipeline workflow. Follow these steps to run your workflow:

1. **Define a Workflow**: Create a workflow definition using a JSON configuration file. You can use the provided example as a starting point.

2. **Save the Workflow**: Save your workflow definition to a file, e.g., `my_workflow.json`.

3. **Run the Workflow**: Open your terminal or command prompt and use the `pipegenex` CLI tool to run your workflow by providing the path to the workflow JSON file as an argument:


## Defining Workflow
A typical Bioinformatic pipeline workflow is defined through a JSON configuration file. Here's an example of a workflow definition:

```json

{
    "name": "My First RNASeq Workflow",
    "description": "This is my first RnaSeq workflow in my custom pipeline generator...",
    "working_directory": "~/home/bio/pipeline_2",
    "variables": [
        {
            "key": "samtools_d",
            "value": "~/usr/local/bin/samtools_d"
        },
        {
            "key": "thread_1",
            "value": "50"
        },
        {
            "key": "input_sampleID",
            "value": "human_te_genome.fasta.gz"
        },
        {
            "key": "output_name_key",
            "value": "human_te_genome_rna_seq_out"
        }
    ],
    "tasks": [
        {
            "id": "1",
            "command": "${samtools_d}samtools view -F 14 -b -@ $thread_1 $list_tmp1[0] >${input_sampleID}_m_ERV.bam",
            "description": "Align Reads"
        },
        {
            "id": "2",
            "command": "fastqc ",
            "description": "Quality Control"
        },
        {
            "id": "3",
            "command": "${samtools_d}samtools merge -f ${input_sampleID}_m2.bam ${input_sampleID}_m_ERV.bam ${input_sampleID}_m1_ERV.bam",
            "description": "Merge BAMs"
        },
        {
            "id": "4",
            "command": "python3 ./Scripts/run_python_analysis_script.py ./outputs/${input_sampleID}.bam ./outputs/${output_name_key}.csv",
            "description": "Analyze RnaSeq Outputs"
        },
        {
            "id": "5",
            "command": "climailer pipelineresults@website.org ./outputs/${output_name_key}.csv",
            "description": "Send Output via Email"
        },
        {
            "id": "6",
            "command": "echo 'Welcome to Jumanji'",
            "description": "Sleep for 10 seconds"
        }
    ],
    "run_sequence": [
        "1", "2", "3", "4", "5,6" 
    ]
}


```


In this JSON configuration, you define your workflow's name, description, working directory, variables, tasks, and run sequence. The command field in each task specifies the command to execute, with variables indicated by ${variable_name} syntax.

### Run Sequence
Something worthy of note is the fact that you can define run_sequence for all the tasks in your workflow. Not only that, you can also tell the Engine to run some particular tasks in parallel say tasks with ids 2, 3 are independent of each other, then while defining your run sequence, you define your run sequence as follows:
```json
"run_sequence": [
        "1", "2,3", "4", "5", "6" 
    ]
```

** This means, start running from task 1, run tasks 2 and 3 in parallel, then run task 4, then task 5, and lastly task 6.**


## Running Workflows
After defining your workflow, you can run it using the **pipegenex** CLI tool. Simply provide the path to your workflow JSON file as an argument:
```bash
pipegenex run my_workflow.json
```
**pipengenex** will execute the tasks in the specified sequence, utilizing any available specified parallelization instruction set in run_sequence for optimal performance.

## Report and Error Handling
Upon completing the workflow, **pipengenex** generates a report (report.txt) if the run was successful. In case of errors during execution, an errors.txt file is generated in the working directory. These reports help you monitor the progress and diagnose issues in your pipeline.

## Examples
To see more examples and advanced usage of pipengenex, please refer to the [Examples](https://github.com/propenster/pipegenex/examples) directory in the project repository.

## Contributing
Contributions to pipengenex are welcome! If you'd like to contribute, please check out our [Contribution Guidelines](https://github.com/propenster/pipengenex/contributing.md).

## License
pipengenex is open-source and available under the [MIT License](https://github.com/propenster/pipengenex/license.md). Feel free to use, modify, and distribute it according to the terms of the license.
