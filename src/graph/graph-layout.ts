/**
 * Pure helpers for the force-directed graph layout.
 *
 * Kept free of Svelte reactivity on purpose: the layout is mutated on every
 * animation frame, so it must never be tracked by an `$effect`.
 */

import type { GraphNode } from '../tauri/bridge';

/** A graph node with the physics state used by the force simulation. */
export interface SimNode extends GraphNode {
  x: number;
  y: number;
  vx: number;
  vy: number;
  radius: number;
}

/**
 * Radius of a node, growing with its number of connections (capped).
 *
 * @param connections - Number of edges touching the node.
 */
export function nodeRadius(connections: number): number {
  return 8 + Math.min(connections * 2, 16);
}

/**
 * Rebuilds the simulation nodes after the visible set changed (filters, search...).
 *
 * Nodes that were already on screen keep their position and velocity so the graph
 * does not jump; new nodes are placed on a ring around the center.
 *
 * @param rawNodes - Nodes that must be visible now.
 * @param previous - Simulation nodes currently on screen.
 * @param width - Canvas width in pixels.
 * @param height - Canvas height in pixels.
 * @param random - Random source in [0, 1) (injectable for tests).
 */
export function mergeSimNodes(
  rawNodes: readonly GraphNode[],
  previous: readonly SimNode[],
  width: number,
  height: number,
  random: () => number = Math.random
): SimNode[] {
  const existingMap = new Map(previous.map((n) => [n.id, n]));

  return rawNodes.map((rn, idx) => {
    const existing = existingMap.get(rn.id);
    const radius = nodeRadius(rn.connections_count);

    if (existing) {
      return { ...rn, x: existing.x, y: existing.y, vx: existing.vx, vy: existing.vy, radius };
    }

    const angle = (idx / rawNodes.length) * 2 * Math.PI;
    const ring = 120 + random() * 150;
    return {
      ...rn,
      x: width / 2 + Math.cos(angle) * ring,
      y: height / 2 + Math.sin(angle) * ring,
      vx: 0,
      vy: 0,
      radius,
    };
  });
}
