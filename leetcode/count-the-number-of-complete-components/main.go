package main

func main() {}

func countCompleteComponents(n int, edges [][]int) int {
	adj := make([][]int, n)
	add := func(from, to int) {
		if len(adj[from]) == 0 {
			adj[from] = make([]int, 0)
		}
		adj[from] = append(adj[from], to)
	}

	for _, e := range edges {
		add(e[0], e[1])
		add(e[1], e[0])
	}

	colors := make([]int, n)
	color := 1

	verticeCount := map[int]int{}
	for v := range n {
		if colors[v] > 0 {
			continue
		}

		q := []int{v}
		for len(q) > 0 {
			v := q[0]
			q = q[1:]
			if colors[v] > 0 {
				continue
			}

			colors[v] = color
			verticeCount[color]++

			q = append(q, adj[v]...)
		}
		color++
	}

	edgeCount := map[int]int{}
	for _, e := range edges {
		c := colors[e[0]]
		edgeCount[c]++
	}

	ans := 0
	for c, vc := range verticeCount {
		ec := edgeCount[c]
		if 2*ec == vc*(vc-1) {
			ans++
		}
	}

	return ans
}
