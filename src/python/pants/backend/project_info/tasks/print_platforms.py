# coding=utf-8
# Copyright 2014 Pants project contributors (see CONTRIBUTORS.md).
# Licensed under the Apache License, Version 2.0 (see LICENSE).
from pants.backend.python.tasks.python_execution_task_base import PythonExecutionTaskBase

class PrintPlatforms(PythonExecutionTaskBase):

  @classmethod
  def register_options(cls, register):
    super(PrintPlatforms, cls).register_options(register)


  def print_platforms(self):
    def info(message):
      self.context.log.info(message)

    platforms = self.context.options.for_scope("python-setup").platforms
    info("Plaforms for python-setup")
    for platform in platforms:
      info("Platform: {}".format(platform))

  def execute(self):
    super(PrintPlatforms, self).execute()
    self.print_platforms()
