#include "inc/uv.inc"
#include "inc/rt.inc"

uniform restrict writeonly image2D outputTex;
uniform vec4 outputTex_size;

layout(std430) buffer constants {
    mat4 clip_to_view;
    mat4 view_to_world;
	uint frame_idx;
};

vec3 perp_hm(vec3 u) {
    vec3 a = abs(u);
    vec3 v;
    if (a.x <= a.y && a.x <= a.z)
        v = vec3(0, -u.z, u.y);
    else if (a.y <= a.x && a.y <= a.z)
        v = vec3(-u.z, 0, u.x);
    else
        v = vec3(-u.y, u.x, 0);
    return v;
}

uint hash(uint x) {
	x += (x << 10u);
	x ^= (x >>  6u);
	x += (x <<  3u);
	x ^= (x >> 11u);
	x += (x << 15u);
	return x;
}

float rand_float(uint h) {
	const uint mantissaMask = 0x007FFFFFu;
	const uint one = 0x3F800000u;

	h &= mantissaMask;
	h |= one;

	float  r2 = uintBitsToFloat( h );
	return r2 - 1.0;
}

// https://www.shadertoy.com/view/4t2SDh
float n2rand_faster(float nrnd0)
{
    // Convert uniform distribution into triangle-shaped distribution.
    float orig = nrnd0 * 2.0 - 1.0;
    nrnd0 = orig * inversesqrt(abs(orig));
    nrnd0 = max(-1.0, nrnd0); // Nerf the NaN generated by 0*rsqrt(0). Thanks @FioraAeterna!
    return nrnd0 - sign(orig) + 0.5;
}

layout (local_size_x = 8, local_size_y = 8) in;
void main() {
	ivec2 pix = ivec2(gl_GlobalInvocationID.xy);
    vec2 uv = get_uv(outputTex_size);
    vec4 ray_origin_cs = vec4(uv_to_cs(uv), 1.0, 1.0);
    vec4 ray_origin_ws = view_to_world * (clip_to_view * ray_origin_cs);
    ray_origin_ws /= ray_origin_ws.w;

    vec4 ray_dir_cs = vec4(uv_to_cs(uv), 0.0, 1.0);
    vec4 ray_dir_ws = view_to_world * (clip_to_view * ray_dir_cs);
    vec3 v = -normalize(ray_dir_ws.xyz);

    Ray r;
    r.o = ray_origin_ws.xyz;
    r.d = -v;

	vec4 col = vec4(r.d * 0.5 + 0.5, 1.0) * 0.5;

	uint seed0 = hash(hash(frame_idx ^ hash(pix.x)) ^ pix.y);

    RtHit hit;
    if (raytrace(r, hit)) {
        Triangle tri = unpack_triangle(bvh_triangles[hit.tri_idx]);
        vec3 normal = normalize(cross(tri.e0, tri.e1));

#if 1
		vec3 t0 = normalize(perp_hm(normal));
		vec3 t1 = cross(t0, normal);
		vec3 l = normalize(vec3(1, 1, -1));

		{
			uint seed1 = hash(seed0);
			float theta = rand_float(seed0) * 6.28318530718;
			float r = 0.5 * sqrt(rand_float(seed1));
			l += r * t0 * cos(theta) + r * t1 * sin(theta);
		}

		l = normalize(l);
#else
        vec3 l = normalize(vec3(1, 1, -1));
#endif

        float ndotl = max(0.0, dot(normal, l));
        uint iter = hit.debug_iter_count;

        r.o += r.d * hit.t;
        r.o -= r.d * 1e-4 * length(r.o);
        r.d = l;
        bool shadowed = raytrace(r, hit);
        //iter = hit.debug_iter_count;

		const float ambient = 0.1;

        col.rgb = ndotl.xxx * 0.8 * (shadowed ? 0.0 : 1.0) + mix(normal * 0.5 + 0.5, 0.5.xxx, 0.5.xxx) * ambient;

        //col.rgb *= 0.1;
    }

	{
		float rnd = rand_float(seed0);
		col.rgb += (rnd - 0.5) * 0.05;
	}

    //col.r = hit.debug_iter_count * 0.01;
    col.a = 1;

	imageStore(outputTex, pix, col);
}
