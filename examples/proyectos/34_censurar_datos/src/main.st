// Ocultar datos en un texto
// Sustituye un identificador conocido y una dirección de ejemplo antes de compartir un mensaje.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

variable original = "Pedido 1234; contacto ana@example.test; cliente Ana";
variable publico = reemplazar(original,"ana@example.test","[correo oculto]");
publico = reemplazar(publico,"Pedido 1234","Pedido [oculto]");
mostrar(publico);
mostrar("Original conservado: " + (buscar(original,"1234") >= 0));
