struct EntradaVertice {
    @location(0) posicion: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}
struct SalidaVertice {
    @builtin(position) posicion: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) normal: vec3<f32>,
}
@group(0) @binding(0) var textura: texture_2d<f32>;
@group(0) @binding(1) var muestreador: sampler;

@vertex fn vs_main(entrada: EntradaVertice, @builtin(instance_index) instancia: u32) -> SalidaVertice {
    var salida: SalidaVertice;
    var desplazamiento = vec3<f32>(-0.55,0.0,0.2);
    if instancia == 1u { desplazamiento = vec3<f32>(0.0,0.15,0.0); }
    if instancia == 2u { desplazamiento = vec3<f32>(0.55,0.0,0.4); }
    salida.posicion = vec4<f32>(entrada.posicion + desplazamiento, 1.0);
    salida.uv = entrada.uv;
    salida.normal = entrada.normal;
    return salida;
}
@fragment fn fs_main(entrada: SalidaVertice) -> @location(0) vec4<f32> {
    let base = textureSample(textura, muestreador, entrada.uv).rgb;
    let n = normalize(entrada.normal);
    let luz = normalize(vec3<f32>(0.4, 0.7, 1.0));
    let vista = vec3<f32>(0.0, 0.0, 1.0);
    let mitad = normalize(luz + vista);
    let difusa = max(dot(n,luz),0.0);
    let rugosidad = 0.45;
    let metalico = 0.15;
    let brillo = pow(max(dot(n,mitad),0.0),mix(128.0,4.0,rugosidad));
    let f0 = mix(vec3<f32>(0.04),base,vec3<f32>(metalico));
    let color = base*(0.08 + difusa*(1.0-metalico)) + f0*brillo;
    return vec4<f32>(color, 1.0);
}
