# coding=utf-8
# Copyright 2014 Pants project contributors (see CONTRIBUTORS.md).
# Licensed under the Apache License, Version 2.0 (see LICENSE).

from __future__ import absolute_import, division, print_function, unicode_literals

import os

from pants.task.task import Task
from pants.goal.goal import Goal
from pants.goal.task_registrar import TaskRegistrar as task

def register_goals():
  Goal.by_name("new-goal")
  task(name="taskk", action=HelloTask).install('new-goal')

class HelloTask(Task):
  def execute(self):
    print('Hello World')