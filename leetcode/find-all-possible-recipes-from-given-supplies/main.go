package main

import "fmt"

func main() {
	recipes := []string{"bread", "sandwich", "burger"}
	ingredients := [][]string{{"yeast", "flour"}, {"bread", "meat"}, {"sandwich", "meat", "bread"}}
	supplies := []string{"yeast", "flour", "meat"}
	ans := findAllRecipes(recipes, ingredients, supplies)
	fmt.Println(ans)
}

func findAllRecipes(recipes []string, ingredients [][]string, supplies []string) []string {
	ingredientsRev := make(map[string][]string)
	for i := range ingredients {
		for _, ingredient := range ingredients[i] {
			if len(ingredientsRev[ingredient]) == 0 {
				ingredientsRev[ingredient] = make([]string, 1)
			}
			ingredientsRev[ingredient] = append(ingredientsRev[ingredient], recipes[i])
		}
	}

	recipesCount := make(map[string]int)
	for i, r := range recipes {
		recipesCount[r] = len(ingredients[i])
	}

	seen := make(map[string]bool)

	queue := make([]string, 0)
	queue = append(queue, supplies...)

	for len(queue) > 0 {
		ingredient := queue[0]
		queue = queue[1:]
		if seen[ingredient] {
			continue
		}
		seen[ingredient] = true
		for _, recipe := range ingredientsRev[ingredient] {
			_, ok := recipesCount[recipe]
			if !ok {
				continue
			}
			recipesCount[recipe]--
			if recipesCount[recipe] == 0 {
				queue = append(queue, recipe)
				delete(recipesCount, recipe)
			}
		}
	}

	ans := make([]string, 0)
	for _, s := range recipes {
		if seen[s] {
			ans = append(ans, s)
		}
	}

	return ans
}
