#pragma once

#include <cstdlib>

template<typename T>
class List
{
public:
  List() {
    this->m_size = 0;
    this->m_capacity = 10;
    this->m_ptr = (T*)malloc(this->m_capacity);
  }

  ~List() {
    free(this->m_ptr);
    this->m_ptr = nullptr;
  }

  /// Adds a value to the list.
  void add(T value) {
    ensureCapacity(m_size + 1);
    this->m_ptr[m_size++] = value;
  }

  T &operator[](unsigned long index) {
    assert(index < this->m_size, "index out of bounds");
    return *this->m_ptr[index];
  }

  unsigned long size() { return this->m_size; }

protected:

  void ensureCapacity(unsigned long capacity) {
    if (this->m_capacity < capacity) {
      this->m_ptr = (T*)realloc(this->m_ptr, capacity);
      assert(this->m_ptr, "failed to reallocate");

      this->m_capacity = capacity;
    }
  }

private:
  T *m_ptr;
  unsigned long m_capacity;
  unsigned long m_size;
};

