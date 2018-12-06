class Node(object):
  def __init__(self, name, outgoing_edges, incoming_edges):
    self.name = name
    self.outgoing_edges = outgoing_edges
    self.incoming_edges = incoming_edges

  def set_of_outgoing_edges(self):
    return set(self.outgoing_edges.keys())
  
class Graph(object):
  def __init__(self, outgoing_edges, incoming_edges):
    self.graph = {}
    for key, value in outgoing_edges.items():
      self.graph[key] = Node(key, outgoing_edges.get(key), incoming_edges.get(key))

  # graph.dfs("A", None, process_node)
  def dfs(self, start, process_node, visited=None):
    if visited is None:
      visited = set()
    visited.add(start)
    for next in self.graph[start].set_of_outgoing_edges() - visited:
      removed = self.dfs(next, process_node, visited)
    currently_removed = process_node(start)
    return removed.union(currently_removed)

