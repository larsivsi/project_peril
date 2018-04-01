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
PointLight light = PointLight(15.0, vec3(1.0, 1.0, 1.0));

void main()
{
	vec3 color = vec3(0.0);
	vec3 texcolor = texture(color_tex, tex_uv).rgb;
	// for each light
	for (uint i = 0; i < 1u; i++)
	{
		// Set up phong variables
		if (length(worldspace_lightdir) > light.radius)
			continue;
		vec3 L_r = worldspace_lightdir / light.radius;

		vec3 L = normalize(tangentspace_lightdir);
		vec3 V = normalize(tangentspace_eyedir);
		// Look up the normal and move it from [0,1] to [-1, 1]
		vec3 N = 2.0 * texture(normal_tex, tex_uv).rgb - 1.0;
		vec3 R = normalize(reflect(-L, N));

		// Diffuse
		float diffuse = max(dot(L, N), 0.0);

		// Specular
		float specular = pow(max(dot(R, V), 0.0), 5.0);

		// Attenuation
		float attenuation = max(0.0, 1.0 - dot(L_r,L_r));

		color += (texcolor * light.color * diffuse * attenuation) + (light.color * specular * attenuation);
	}
	fragColor = color;
}
