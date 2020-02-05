# Copyright 2017 Pants project contributors (see CONTRIBUTORS.md).
# Licensed under the Apache License, Version 2.0 (see LICENSE).
from pants.contrib.ideexport.ide_export import IdeExport
from pants.goal.task_registrar import TaskRegistrar as task



def register_goals():
  task(name='ide-export', action=IdeExport).install()
