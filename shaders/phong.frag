#version 450
#extension GL_ARB_separate_shader_objects : enable
precision highp float;

layout(location = 0) in vec3 viewspace_pos;
layout(location = 1) in vec3 viewspace_normal;
layout(binding = 0) uniform MatrixBlock {
	mat4 v;
} Matrices;
layout(location = 0) out vec3 fragColor;

struct PointLight {
	vec3 pos;
	float radius;
	vec3 color;
};

//hardcoded for now
PointLight light = PointLight(vec3(0.0, 0.0, 0.0), 6.0, vec3(1.0, 1.0, 1.0));

void main()
{
	vec3 color = vec3(0.0);
	// for each light
	for (uint i = 0; i < 1u; i++)
	{
		// lightpos is a point, set w to 1.0 and divide it out afterwards
		vec4 viewspace_lightpos4 = Matrices.v * vec4(light.pos, 1.0);
		vec3 viewspace_lightpos = vec3(viewspace_lightpos4) / viewspace_lightpos4.w;

		// Set up phong variables
		vec3 light_dir = viewspace_lightpos - viewspace_pos;
		if (length(light_dir) > light.radius)
			continue;
		vec3 L_r = light_dir / light.radius;

		vec3 L = normalize(light_dir);
		vec3 V = normalize(-viewspace_pos);
		vec3 N = viewspace_normal;
		vec3 R = normalize(reflect(-L, N));

		// Diffuse
		float diffuse = max(dot(L, N), 0.0);

		// Specular
		float specular = pow(max(dot(R, V), 0.0), 5.0);

		// Attenuation
		float attenuation = max(0.0, 1.0 - dot(L_r,L_r));

		color += light.color * (diffuse + specular) * attenuation;
	}
	fragColor = color;
}
