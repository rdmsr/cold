int my_function(void);
int undefined_func(void);

int main(void) {
  undefined_func();
  return my_function();
}
