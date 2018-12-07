import subprocess
import os
from argument_parsing import parse_cli
from process_graph import process_graph

def call_dep_usage(target, output_file):
  command = [
    "./pants",
    "clean-all",
    "dep-usage.jvm",
    "--no-cache-compile-zinc-read",
    "--no-summary",
    "--no-transitive",
    "--output-file={}".format(output_file),
    target
  ]
  subprocess.run(command, check=True)

def call_dependees(target, output_file):
  with open(output_file, "wb") as ofile:
    command = [
      "./pants",
      "dependees",
      "--output-format=json",
      "--no-transitive",
      target
    ]
    subprocess.run(command, check=True, stdout=ofile)

args = parse_cli()
target = args.target
dep_usage_output_path = args.dependencies_file
dependees_output_path = args.direct_dependees_file
enable_strict_deps = args.enable_strict_deps

# Log to notify of eventual mismatches between source & old dep-usage file
if not os.path.exists(dep_usage_output_path):
  call_dep_usage(target, dep_usage_output_path)

if not os.path.exists(dependees_output_path):
  call_dependees(target, dependees_output_path)

process_graph(args)



