// Primer ejemplo ABI 1. No abre archivos, usa Python ni procesa JSON.
importar "nativo";
variable datos = nativo.desde_texto("Hola ñ");
intentar {
    mostrar(nativo.version()); // 1
    mostrar(nativo.tamano(datos)); // 7 bytes; no confundir con caracteres.
    nativo.poner(datos; 0; 104); // 'h'
    mostrar(nativo.texto(datos)); // hola ñ
} finalmente {
    nativo.cerrar(datos); // Libera la ranura incluso si una operación lanza un error.
}
// Los alias del identificador también quedan invalidados después de cerrar.
intentar { nativo.obtener(datos; 0); }
capturar error { mostrar(error); }
