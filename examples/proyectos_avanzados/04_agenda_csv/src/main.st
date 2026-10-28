importar "archivos";
importar "io";
importar "json";
// Escapa comillas CSV; protege prefijos que una hoja de cálculo podría tratar como fórmulas.
funcion celda(texto: cadena) -> cadena {
    variable seguro = texto;
    si longitud(texto) > 0 {
        si buscar("=+-@", texto[0]) >= 0 { seguro = "'" + texto; }
    }
    retornar "\"" + reemplazar(seguro, "\"", "\"\"") + "\"";
}
variable nombre = io.leer_linea("Nombre: ");
variable correo = io.leer_linea("Correo: ");
si nombre == "" o buscar(correo, "@") < 1 { lanzar "Nombre y correo válidos requeridos"; }
archivos.escribir_texto("contacto.csv", "nombre,correo\n" + celda(nombre) + "," + celda(correo) + "\n", falso, "utf-8");
archivos.escribir_texto("contacto.json", json.objeto(["nombre", "correo"], [json.codificar(nombre), json.codificar(correo)]), falso, "utf-8");
mostrar("Contacto exportado en CSV y JSON");
