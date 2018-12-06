class Node(object):
  def __init__(self, name, outgoing_edges, incoming_edges):
    self.name = name
    self.outgoing_edges = outgoing_edges
    self.incoming_edges = set(incoming_edges)

  def set_of_outgoing_edges(self):
    return set(self.outgoing_edges.keys())
  
class Graph(object):
  def __init__(self, outgoing_edges, incoming_edges):
    self.graph = {}
    for key, value in outgoing_edges.items():
      self.graph[key] = Node(key, outgoing_edges.get(key, set()), incoming_edges.get(key, set()))

  # graph.dfs("A", None, process_node)
  def dfs(self, start, process_node, visited=None):
    if visited is None:
      visited = set()
    visited.add(start)
    removed = set()
    to_visit = self.graph[start].set_of_outgoing_edges() - visited
    while to_visit:
      next = to_visit.pop()
      removed, visited_by_children = self.dfs(next, process_node, visited)
      to_visit = to_visit - visited_by_children
    removed_by_this_node = process_node(self.graph[start], set(self.graph.keys()))
    return removed.union(removed_by_this_node), visited

