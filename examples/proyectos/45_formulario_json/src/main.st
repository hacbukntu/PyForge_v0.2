// Exportar formulario a JSON
// Valida edad y serializa nombre, edad y consentimiento.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

importar "json";
importar "io";
mostrar("Nombre y edad:");
variable nombre = entrada("");
variable edad = entero(entrada(""));
si longitud(nombre) == 0 o edad < 0 o edad > 120 { lanzar "Formulario inválido"; }
variable claves = ["nombre","edad","activo"];
variable valores = [json.codificar(nombre),json.de_entero(edad),json.de_booleano(verdadero)];
variable documento = json.objeto(claves,valores);
io.escribir_texto("formulario.json",documento);
mostrar(documento);
