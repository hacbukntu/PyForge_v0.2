// Configuración JSON tipada
// Carga opciones de aplicación y convierte explícitamente sus valores.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

importar "json";
importar "io";
variable config = io.leer_texto("config.json");
json.comprobar(config);
variable nombre = json.decodificar(json.obtener(config,"nombre"));
variable limite = json.a_entero(json.obtener(config,"limite"));
variable activo = json.a_booleano(json.obtener(config,"activo"));
si limite < 1 o limite > 100 { lanzar "Límite de configuración inválido"; }
mostrar(nombre + ": " + limite);
mostrar("Activo: " + activo);
