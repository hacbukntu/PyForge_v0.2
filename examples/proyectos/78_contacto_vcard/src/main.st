// Tarjeta de contacto vCard
// Exporta un contacto ficticio como tarjeta de texto estructurado.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

importar "io";
// Datos ficticios controlados: el ejemplo no acepta campos con saltos o separadores.
variable nombre = "Ana";
variable apellido = "Perez";
variable telefono = "555-0101";
variable tarjeta = "BEGIN:VCARD\r\nVERSION:3.0\r\nN:" + apellido + ";" + nombre + ";;;\r\nFN:" + nombre + " " + apellido + "\r\nTEL:" + telefono + "\r\nEMAIL:ana@example.test\r\nEND:VCARD\r\n";
io.escribir_texto("contacto.vcf",tarjeta);
mostrar("Tarjeta generada");
