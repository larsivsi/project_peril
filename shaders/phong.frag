#version 450
#extension GL_ARB_separate_shader_objects : enable
precision highp float;

layout(location = 0) in vec3 tangentspace_eyedir;
layout(location = 1) in vec3 worldspace_lightdir;
layout(location = 2) in vec3 tangentspace_lightdir;
layout(location = 3) in vec2 tex_uv;

layout(set = 0, binding = 0) uniform sampler2D color_tex;
layout(set = 0, binding = 1) uniform sampler2D normal_tex;

layout(location = 0) out vec3 fragColor;

struct PointLight {
	float radius;
	vec3 color;
};

//hardcoded for now
PointLight light = PointLight(1000.0, vec3(1.0, 1.0, 1.0));

void main()
{
	vec3 color = vec3(0.0);
	vec3 texcolor = texture(color_tex, tex_uv).rgb;
	// for each light
	for (uint i = 0; i < 1u; i++)
	{
		// Check distance and calculate attenuation
		if (length(worldspace_lightdir) > light.radius)
			continue;
		vec3 L_div_r = worldspace_lightdir / light.radius;
		float attenuation = max(1.0 - dot(L_div_r, L_div_r), 0.0);

		// Set up phong variables
		vec3 L = normalize(tangentspace_lightdir);
		// Look up the normal
		vec3 normal = texture(normal_tex, tex_uv).rgb;
		// Flip y-value from top left to bottom left
		normal.g = 1.0 - normal.g;
		// Move normal it from [0,1] to [-1, 1]
		vec3 N = normalize(2.0 * normal - 1.0);

		float lambertian = max(dot(L, N), 0.0);
		float specular = 0.0;

		if (lambertian > 0.0)
		{
			vec3 V = normalize(tangentspace_eyedir);
			vec3 R = normalize(reflect(-L, N));
			specular = pow(max(dot(R, V), 0.0), 50.0);
		}

		// Diffuse
		color += texcolor * lambertian * light.color * attenuation;

		// Specular
		color += specular * light.color * attenuation;
	}
	fragColor = color;
}
