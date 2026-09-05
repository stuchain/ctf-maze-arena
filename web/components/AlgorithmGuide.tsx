import { Panel, PanelHeader } from '@/components/ui/Primitives';

const SOLVERS = [
  { name: 'BFS', mechanism: 'Expands equal-distance waves with a queue.', time: 'O(V + E)', space: 'O(V)', guarantee: 'Complete and shortest on unit-cost mazes.', behavior: 'Broad, even exploration.' },
  { name: 'DFS', mechanism: 'Follows one branch deeply with a stack.', time: 'O(V + E)', space: 'O(V)', guarantee: 'Complete on this finite graph; not shortest.', behavior: 'Narrow dives and long paths.' },
  { name: 'A*', mechanism: 'Orders cells by known cost plus Manhattan estimate.', time: 'O(V + E) worst case', space: 'O(V)', guarantee: 'Complete and shortest with this admissible heuristic.', behavior: 'Goal-directed exploration.' },
  { name: 'DP Keys', mechanism: 'Runs BFS over (cell, key-set) states.', time: 'O((V + E) · 2ᵏ)', space: 'O(V · 2ᵏ)', guarantee: 'Complete and shortest in the expanded state graph.', behavior: 'Revisits cells with different inventories.' },
];

export function AlgorithmGuide() {
  return <Panel className="algorithm-guide"><PanelHeader eyebrow="Learn from the race" title="Algorithm Field Guide" description="Complexities use V cells, E passages, and k keys. Visible metrics connect the theory to this exact maze." />
    <div className="algorithm-cards">{SOLVERS.map((solver) => <article key={solver.name}><h3>{solver.name}</h3><p>{solver.mechanism}</p><dl><div><dt>Time</dt><dd>{solver.time}</dd></div><div><dt>Space</dt><dd>{solver.space}</dd></div><div><dt>Guarantee</dt><dd>{solver.guarantee}</dd></div><div><dt>Signature</dt><dd>{solver.behavior}</dd></div></dl></article>)}</div>
    <details><summary>How the generators shape a race</summary><p><strong>Kruskal</strong> shuffles grid edges and joins separate components. <strong>Prim</strong> grows from a randomized frontier; this implementation does not sample spanning trees uniformly. <strong>DFS backtracker</strong> carves long corridors through recursive depth-first exploration. All three produce deterministic perfect mazes for a fixed seed.</p></details>
  </Panel>;
}
