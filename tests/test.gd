extends Node

func test(conf: Dictionary, inputs: int, outputs: int, fitness: Callable, try: PackedFloat64Array, nam: String, working: PackedByteArray):
	var neat := NEAT.new()
	neat.inputs = inputs
	neat.outputs = outputs
	neat.fitness_fn = fitness

	neat.reporter_fn = func(i: int) -> void: print("generation %d." % i)
	var cfg := Configuration.new()
	cfg.population_size = conf.population_size
	cfg.max_generations = conf.max_generations
	cfg.mutation_rate = conf.mutation_rate
	cfg.set_fitness_goal(conf.fitness_goal)
	cfg.node_cost = conf.node_cost
	cfg.connection_cost = conf.connection_cost
	cfg.compatibility_threshold = conf.compatability_threshold
	neat.configuration = cfg
	var res := neat.start()
	var test_res: PackedFloat64Array = res.network.forward_pass(try) if res else PackedFloat64Array()
	var expected
	if working:
		var wnet := Network.new()
		wnet.from_bytes(working)
		expected = wnet.forward_pass(try)
	print_rich("test %s: %s
	sent %s, got %s
	expected: %s
	network: %s
	" % [nam, "[color=cb3a37]failed[/color]" if not (expected == test_res) else "[color=#36be4e]passed[/color]",  test, test_res, expected, res.network.to_bytes().hex_encode() if res else "null"])


func _ready() -> void:
	seed(0)
	test(
		{population_size=150, max_generations=100, mutation_rate=0.75, fitness_goal=0.9099, node_cost=0.01,connection_cost=0.01,compatability_threshold=3.0},
		2,
		1,
	(func xor_fit(network: Network) -> float:
		var error := 0.0
		var input: Array[PackedFloat64Array] = [
			[0.0, 0.0],
			[0.0, 1.0],
			[1.0, 0.0],
			[1.0, 1.0],
		]
		for z in input:
			var results := network.forward_pass(z)
			var result := results[0]
			error += pow((int(z[0])^int(z[1])) - result, 2)
		return 1. / (1. + error)),
		[0, 1],
		"xor",
		[]
	)
	get_tree().quit()
