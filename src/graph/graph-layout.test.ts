import { describe, it, expect } from 'vitest';
import { mergeSimNodes, nodeRadius, type SimNode } from './graph-layout';
import type { GraphNode } from '../tauri/bridge';

function node(id: string, connections = 0): GraphNode {
  return {
    id,
    title: id,
    kind: 'character',
    summary: null,
    tags: [],
    status: null,
    mention_count: 0,
    connections_count: connections,
    is_orphan: connections === 0,
  };
}

function sim(id: string, x: number, y: number): SimNode {
  return { ...node(id), x, y, vx: 1.5, vy: -2, radius: 8 };
}

describe('graph layout', () => {
  it('grows the radius with connections and caps it', () => {
    expect(nodeRadius(0)).toBe(8);
    expect(nodeRadius(3)).toBe(14);
    expect(nodeRadius(100)).toBe(24);
  });

  it('drops filtered-out nodes immediately', () => {
    const previous = [sim('a', 10, 10), sim('b', 20, 20), sim('c', 30, 30)];
    const result = mergeSimNodes([node('a'), node('c')], previous, 800, 600);
    expect(result.map((n) => n.id)).toEqual(['a', 'c']);
  });

  it('keeps position and velocity of nodes that stay visible', () => {
    const previous = [sim('a', 10, 12)];
    const [kept] = mergeSimNodes([node('a', 4)], previous, 800, 600);
    expect(kept.x).toBe(10);
    expect(kept.y).toBe(12);
    expect(kept.vx).toBe(1.5);
    expect(kept.vy).toBe(-2);
    expect(kept.radius).toBe(nodeRadius(4));
  });

  it('places nodes that reappear on a ring around the center', () => {
    const [fresh] = mergeSimNodes([node('z')], [], 800, 600, () => 0);
    // angle 0, ring 120 -> (400 + 120, 300)
    expect(fresh.x).toBeCloseTo(520);
    expect(fresh.y).toBeCloseTo(300);
    expect(fresh.vx).toBe(0);
    expect(fresh.vy).toBe(0);
  });

  it('returns an empty layout when nothing is visible', () => {
    expect(mergeSimNodes([], [sim('a', 1, 1)], 800, 600)).toEqual([]);
  });
});
