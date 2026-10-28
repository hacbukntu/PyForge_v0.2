// Geometría de rectángulos
// Usa estructura y métodos agregados con implementar para calcular área y perímetro.
// Consulta README.md para ejecutar, modificar y probar este proyecto.

estructura Rectangulo { variable ancho: entero; variable alto: entero; }
implementar Rectangulo {
    // Las dimensiones se validan al calcular para mostrar una invariante explícita.
    funcion area() -> entero {
        si yo.ancho < 0 o yo.alto < 0 { lanzar "Dimensión negativa"; }
        retornar yo.ancho * yo.alto;
    }
    // Perímetro en las mismas unidades que las dimensiones.
    funcion perimetro() -> entero { retornar 2*(yo.ancho+yo.alto); }
}
variable figura = nuevo Rectangulo(4,3);
mostrar("Área: " + figura.area()); mostrar("Perímetro: " + figura.perimetro());
