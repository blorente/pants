# Copyright 2017 Pants project contributors (see CONTRIBUTORS.md).
# Licensed under the Apache License, Version 2.0 (see LICENSE).

import logging

from pants.task.task import Task


logger = logging.getLogger(__name__)


class IdeExport(Task):

  options_scope = 'ide-export'

  @classmethod
  def subsystem_dependencies(cls):
    return super().subsystem_dependencies()

  @classmethod
  def register_options(cls, register):
    register('--j', type=str, help='The dependency or dependencies to add')
    register('--remove-dependencies', type=str, help='The dependency or dependencies to remove')
    register('--command', type=str, help='A custom buildozer command to execute')

  def execute(self):
    
    pass
