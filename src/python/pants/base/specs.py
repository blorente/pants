# Copyright 2014 Pants project contributors (see CONTRIBUTORS.md).
# Licensed under the Apache License, Version 2.0 (see LICENSE).

import os
import re
from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import TYPE_CHECKING, Dict, Iterable, Iterator, List, Optional, Sequence, Tuple, cast

from pants.util.collections import assert_single_element
from pants.util.dirutil import fast_relpath_optional, recursive_dirname
from pants.util.filtering import create_filters, wrap_filters
from pants.util.memo import memoized_property
from pants.util.meta import frozen_after_init


if TYPE_CHECKING:
  from pants.engine.mapper import AddressFamily, AddressMapper


class AddressSpec(ABC):
  """Represents address selectors as passed from the command line.

  Supports `Single` target addresses as well as `Sibling` (:) and `Descendant` (::) selector forms.

  Note: In general, 'spec' should not be a user visible term, it is usually appropriate to
  substitute 'address' for a spec resolved to an address, or 'address selector' if you are
  referring to an unresolved spec string.
  """

  @abstractmethod
  def to_spec_string(self) -> str:
    """Returns the normalized string representation of this address spec."""

  class AddressFamilyResolutionError(Exception):
    pass

  @abstractmethod
  def matching_address_families(
    self, address_families_dict: Dict[str, "AddressFamily"],
  ) -> List["AddressFamily"]:
    """Given a dict of (namespace path) -> AddressFamily, return the values matching this address
    spec.

    :raises: :class:`AddressSpec.AddressFamilyResolutionError` if no address families matched this spec.
    """

  @classmethod
  def address_families_for_dir(
    cls, address_families_dict: Dict[str, "AddressFamily"], spec_dir_path: str
  ) -> List["AddressFamily"]:
    """Implementation of `matching_address_families()` for address specs matching at most
    one directory."""
    maybe_af = address_families_dict.get(spec_dir_path, None)
    if maybe_af is None:
      raise cls.AddressFamilyResolutionError(
        'Path "{}" does not contain any BUILD files.'
        .format(spec_dir_path))
    return [maybe_af]

  class AddressResolutionError(Exception):
    pass

  @abstractmethod
  def address_target_pairs_from_address_families(self, address_families: List["AddressFamily"]):
    """Given a list of AddressFamily, return (address, target) pairs matching this address spec.

    :raises: :class:`SingleAddress._SingleAddressResolutionError` for resolution errors with a
             :class:`SingleAddress` instance.
    :raises: :class:`AddressSpec.AddressResolutionError` if no targets could be found otherwise, if
             the address spec type requires a non-empty set of targets.
    :return: list of (Address, Target) pairs.
    """

  @classmethod
  def all_address_target_pairs(cls, address_families):
    """Implementation of `address_target_pairs_from_address_families()` which does no filtering."""
    addr_tgt_pairs = []
    for af in address_families:
      addr_tgt_pairs.extend(af.addressables.items())
    return addr_tgt_pairs

  @abstractmethod
  def make_glob_patterns(self, address_mapper: "AddressMapper") -> List[str]:
    """Generate glob patterns matching exactly all the BUILD files this address spec covers."""

  @classmethod
  def globs_in_single_dir(cls, spec_dir_path: str, address_mapper: "AddressMapper") -> List[str]:
    """Implementation of `make_glob_patterns()` which only allows a single base directory."""
    return [os.path.join(spec_dir_path, pat) for pat in address_mapper.build_patterns]


@dataclass(frozen=True)
class SingleAddress(AddressSpec):
  """An AddressSpec for a single address."""
  directory: str
  name: str

  def __post_init__(self) -> None:
    if self.directory is None:
      raise ValueError(f'A SingleAddress must have a directory. Got: {self}')
    if self.name is None:
      raise ValueError(f'A SingleAddress must have a name. Got: {self}')

  def to_spec_string(self) -> str:
    return '{}:{}'.format(self.directory, self.name)

  def matching_address_families(
    self, address_families_dict: Dict[str, "AddressFamily"]
  ) -> List["AddressFamily"]:
    return self.address_families_for_dir(address_families_dict, self.directory)

  class _SingleAddressResolutionError(Exception):
    def __init__(self, single_address_family: "AddressFamily", name: str) -> None:
      super().__init__()
      self.single_address_family = single_address_family
      self.name = name

  def address_target_pairs_from_address_families(self, address_families: Sequence["AddressFamily"]):
    """Return the pair for the single target matching the single AddressFamily, or error.

    :raises: :class:`SingleAddress._SingleAddressResolutionError` if no targets could be found for a
             :class:`SingleAddress` instance.
    :return: list of (Address, Target) pairs with exactly one element.
    """
    single_af = assert_single_element(address_families)
    addr_tgt_pairs = [
      (addr, tgt) for addr, tgt in single_af.addressables.items()
      if addr.target_name == self.name
    ]
    if len(addr_tgt_pairs) == 0:
      raise self._SingleAddressResolutionError(single_af, self.name)
    # There will be at most one target with a given name in a single AddressFamily.
    assert(len(addr_tgt_pairs) == 1)
    return addr_tgt_pairs

  def make_glob_patterns(self, address_mapper: "AddressMapper") -> List[str]:
    return self.globs_in_single_dir(self.directory, address_mapper)


@dataclass(frozen=True)
class SiblingAddresses(AddressSpec):
  """An AddressSpec representing all addresses located directly within the given directory."""
  directory: str

  def to_spec_string(self) -> str:
    return f'{self.directory}:'

  def matching_address_families(
    self, address_families_dict: Dict[str, "AddressFamily"],
  ) -> List["AddressFamily"]:
    return self.address_families_for_dir(address_families_dict, self.directory)

  def address_target_pairs_from_address_families(self, address_families: Sequence["AddressFamily"]):
    return self.all_address_target_pairs(address_families)

  def make_glob_patterns(self, address_mapper: "AddressMapper") -> List[str]:
    return self.globs_in_single_dir(self.directory, address_mapper)


@dataclass(frozen=True)
class DescendantAddresses(AddressSpec):
  """An AddressSpec representing all addresses located recursively under the given directory."""
  directory: str

  def to_spec_string(self) -> str:
    return f'{self.directory}::'

  def matching_address_families(
    self, address_families_dict: Dict[str, "AddressFamily"],
  ) -> List["AddressFamily"]:
    return [
      af for ns, af in address_families_dict.items()
      if fast_relpath_optional(ns, self.directory) is not None
    ]

  def address_target_pairs_from_address_families(self, address_families: Sequence["AddressFamily"]):
    addr_tgt_pairs = self.all_address_target_pairs(address_families)
    if len(addr_tgt_pairs) == 0:
      raise self.AddressResolutionError('AddressSpec {} does not match any targets.'.format(self))
    return addr_tgt_pairs

  def make_glob_patterns(self, address_mapper: "AddressMapper") -> List[str]:
    return [os.path.join(self.directory, '**', pat) for pat in address_mapper.build_patterns]


@dataclass(frozen=True)
class AscendantAddresses(AddressSpec):
  """An AddressSpec representing all addresses located recursively _above_ the given directory."""
  directory: str

  def to_spec_string(self) -> str:
    return f'{self.directory}^'

  def matching_address_families(
    self, address_families_dict: Dict[str, "AddressFamily"],
  ) -> List["AddressFamily"]:
    return [
      af for ns, af in address_families_dict.items()
      if fast_relpath_optional(self.directory, ns) is not None
    ]

  def address_target_pairs_from_address_families(self, address_families):
    return self.all_address_target_pairs(address_families)

  def make_glob_patterns(self, address_mapper: "AddressMapper") -> List[str]:
    return [
      os.path.join(f, pattern)
      for pattern in address_mapper.build_patterns
      for f in recursive_dirname(self.directory)
    ]


_specificity = {
  SingleAddress: 0,
  SiblingAddresses: 1,
  AscendantAddresses: 2,
  DescendantAddresses: 3,
  type(None): 99
}


def more_specific(
  address_spec1: Optional[AddressSpec], address_spec2: Optional[AddressSpec]
) -> AddressSpec:
  """Returns which of the two specs is more specific.

  This is useful when a target matches multiple specs, and we want to associate it with
  the "most specific" one, which will make the most intuitive sense to the user.
  """
  # Note that if either of spec1 or spec2 is None, the other will be returned.
  if address_spec1 is None and address_spec2 is None:
    raise ValueError('internal error: both specs provided to more_specific() were None')
  return cast(
    AddressSpec,
    address_spec1 if _specificity[type(address_spec1)] < _specificity[type(address_spec2)] else address_spec2
  )


@frozen_after_init
@dataclass(unsafe_hash=True)
class AddressSpecsMatcher:
  """Contains filters for the output of a AddressSpecs match.

  This class is separated out from `AddressSpecs` to allow for both stuctural equality of the `tags` and
  `exclude_patterns`, and for caching of their compiled forms using `@memoized_property` (which uses
  the hash of the class instance in its key, and results in a very large key when used with
  `AddressSpecs` directly).
  """
  tags: Tuple[str, ...]
  exclude_patterns: Tuple[str, ...]

  def __init__(
    self, tags: Optional[Iterable[str]] = None, exclude_patterns: Optional[Iterable[str]] = None,
  ) -> None:
    self.tags = tuple(tags or [])
    self.exclude_patterns = tuple(exclude_patterns or [])

  @memoized_property
  def _exclude_compiled_regexps(self):
    return [re.compile(pattern) for pattern in set(self.exclude_patterns or [])]

  def _excluded_by_pattern(self, address):
    return any(p.search(address.spec) is not None for p in self._exclude_compiled_regexps)

  @memoized_property
  def _target_tag_matches(self):
    def filter_for_tag(tag):
      return lambda t: tag in [str(t_tag) for t_tag in t.kwargs().get("tags", [])]
    return wrap_filters(create_filters(self.tags, filter_for_tag))

  def matches_target_address_pair(self, address, target):
    """
    :param Address address: An Address to match
    :param HydratedTarget target: The Target for the address.

    :return: True if the given Address/HydratedTarget are included by this matcher.
    """
    return self._target_tag_matches(target) and not self._excluded_by_pattern(address)


@frozen_after_init
@dataclass(unsafe_hash=True)
class AddressSpecs:
  """A collection of `AddressSpec`s representing AddressSpec subclasses, and a AddressSpecsMatcher
  to filter results."""
  dependencies: Tuple[AddressSpec, ...]
  matcher: AddressSpecsMatcher

  def __init__(
    self,
    dependencies: Iterable[AddressSpec],
    tags: Optional[Iterable[str]] = None,
    exclude_patterns: Optional[Iterable[str]] = None,
  ) -> None:
    self.dependencies = tuple(dependencies)
    self.matcher = AddressSpecsMatcher(tags=tags, exclude_patterns=exclude_patterns)

  def __iter__(self) -> Iterator[AddressSpec]:
    return iter(self.dependencies)
