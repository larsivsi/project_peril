#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec3 tangent;
layout(location = 3) in vec3 bitangent;
layout(location = 4) in vec2 tex_uv;

layout(push_constant) uniform MatrixBlock {
	mat4 m;
	mat4 mvp;
} Matrices;

layout(set = 1, binding = 0) uniform ViewMatrixBlock {
       mat4 v;
} ViewMatrix;

layout(location = 0) out vec3 tangentspace_eyedir;
layout(location = 1) out vec3 worldspace_lightdir;
layout(location = 2) out vec3 tangentspace_lightdir;
layout(location = 3) out vec2 interpolated_tex_uv;

vec3 worldspace_lightpos = vec3(0.0, 5.0, 20.0);

void main()
{
	mat4 mv_matrix = ViewMatrix.v * Matrices.m;
	// normal, tangent and bitanget are vectors, set w to 0.0
	vec3 viewspace_normal = vec3(mv_matrix * vec4(normal, 0.0));
	vec3 viewspace_tangent = vec3(mv_matrix * vec4(tangent, 0.0));
	vec3 viewspace_bitangent = vec3(mv_matrix * vec4(bitangent, 0.0));

	// calculate the tangent space matrix
	mat3 TBN = transpose(mat3(viewspace_tangent, viewspace_bitangent, viewspace_normal));

	// position is a point, set w to 1.0 and divide it out afterwards
	vec4 worldspace_pos4 = Matrices.m * vec4(position, 1.0);
	vec3 worldspace_pos = vec3(worldspace_pos4) / worldspace_pos4.w;

	// calculate eyedir and lightdir in tangent space
	vec4 viewspace_pos4 = mv_matrix * vec4(position, 1.0);
	vec3 viewspace_pos = vec3(viewspace_pos4) / viewspace_pos4.w;
	tangentspace_eyedir = TBN * (-viewspace_pos);
	worldspace_lightdir = (worldspace_lightpos - worldspace_pos);
	vec3 viewspace_lightdir = vec3(ViewMatrix.v * vec4(worldspace_lightdir, 0.0));
	tangentspace_lightdir = TBN * viewspace_lightdir;

	// interpolate texture coordinates
	interpolated_tex_uv = tex_uv;

	gl_Position = Matrices.mvp * vec4(position, 1.0);
}
