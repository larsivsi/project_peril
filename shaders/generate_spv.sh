#!/bin/bash

for shader in {*.vert,*.frag}; do
	output_name="${shader/./_}.spv"
	echo "$shader -> $output_name"
	glslangValidator -V $shader -o $output_name
done
