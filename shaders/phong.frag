#version 450
#extension GL_ARB_separate_shader_objects : enable
precision highp float;

layout(location = 0) in vec3 v;
layout(location = 1) in vec3 N;
layout(push_constant) uniform MatrixBlock
{
	mat4 v;
	mat4 mv;
	mat4 mvp;
} Matrices;
layout(location = 0) out vec3 fragColor;

struct PointLight {
	vec3 pos;
	float radius;
	vec3 color;
};

//hardcoded for now
PointLight light = PointLight(vec3(0.0, 0.0, 0.0), 100.0, vec3(1.0, 1.0, 1.0));

void main()
{
	vec3 color = vec3(0.0);
	// for each light
	for (uint i = 0; i < 1u; i++)
	{
		vec4 lp_4 = Matrices.v * vec4(light.pos, 1.0);
		vec3 lp = vec3(lp_4) / lp_4.w;

		//Set up phong variables
		vec3 L = lp - v;
		if (length(L) > light.radius)
			continue;
		vec3 L_r = L / light.radius;
		L = normalize(L);
		vec3 E = normalize(-v);
		vec3 R = normalize(reflect(-L, N));

		//Diffuse
		float diffuse = max(dot(N, L), 0.0);

		//Specular
		float specular = pow(max(dot(R, E), 0.0), 5.0);

		float attenuation = max(0.0, 1.0 - dot(L_r,L_r));

		color += light.color * (diffuse + specular) * attenuation;
	}
	fragColor = color;
}
