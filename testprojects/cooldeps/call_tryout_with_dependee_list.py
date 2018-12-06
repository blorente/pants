import sys
import subprocess

list_file = sys.argv[1]
max_args = 2500

with open(list_file) as f:
  all_targets = [target.strip("\n") for target in f.readlines() if not target.startswith("//:") and not target.startswith("3rdparty")]

  print("All targets = {}".format(len(all_targets)))

  for i in range(0, int(len(all_targets) / max_args)):
    chunk_start = i*max_args
    chunk_end = chunk_start + max_args
    args = all_targets[chunk_start:chunk_end]
    print("Running pants with targets [{}, {}]... len(args) = {}, e.g. {}".format(chunk_start, chunk_end, len(args), args[49]))
    command = ["./scoot/config/scripts/tryout.sh"] + args + [
      "--no-cache-compile-zinc-read",
      "--no-dep-usage-jvm-transitive",
      "--no-dep-usage-jvm-summary",
      # "--dep-usage-jvm-use-cached",
      "-sickle--pants_goals=clean-all,dep-usage.jvm"
    ]
    subprocess.run(command)
    print("DONE")
