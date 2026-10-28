importar "config.st" como config;

// Las variables públicas importadas son de solo lectura para el consumidor.
mostrar(config.version);
mostrar(config.duplicar(21));

// Cada nombre se declara e inicializa antes de utilizarlo.
variable texto = "40";
variable entero_convertido = entero(texto);
variable decimal = flotante(entero_convertido);

// Reasignar conserva el tipo estático.
entero_convertido = entero_convertido + 2;
mostrar(entero_convertido);
mostrar(decimal);
