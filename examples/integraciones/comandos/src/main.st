importar "comandos";
// No usa shell: programa autorizado y argumentos separados.
mostrar(comandos.ejecutar("/usr/bin/printf", "[\"%s\\n\", \"Hola desde un proceso autorizado\"]", 3));
