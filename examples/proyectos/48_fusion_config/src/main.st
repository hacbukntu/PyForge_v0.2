// Combinar configuraciones conocidas
// Combina un nombre base y un límite local, validando las claves esperadas.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

importar "json";
importar "io";
variable base = io.leer_texto("base.json");
variable local = io.leer_texto("local.json");
variable nombre = json.obtener(base,"nombre");
variable limite = json.obtener(local,"limite");
si json.a_entero(limite) <= 0 { lanzar "Límite inválido"; }
variable resultado = json.objeto(["nombre","limite"],[nombre,limite]);
io.escribir_texto("resultado.json", resultado);
mostrar(resultado);
